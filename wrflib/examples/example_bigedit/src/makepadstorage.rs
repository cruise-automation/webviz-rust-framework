// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// A bunch of stuff here doesn't get used in the wasm version.
#![allow(dead_code)]

//use syn::Type;
use crate::builder;
use crate::filetree::*;
use crate::makepadwindow::*;
use bigedit_http::channel::WebSocketChannels;
use bigedit_hub::*;
use bigedit_microserde::*;
use std::collections::HashMap;
use wrflib::*;
use wrflib_components::*;

#[derive(Debug, Clone, SerRon, DeRon)]
pub struct MakepadSettings {
    pub build_on_save: bool,
    pub exec_when_done: bool,
    live_on_self: bool,
    hub_server: HubServerConfig,
    pub builders: HashMap<String, HubBuilderConfig>,
    pub builds: Vec<BuildTarget>,
    sync: HashMap<String, Vec<String>>,
}

impl Default for MakepadSettings {
    fn default() -> Self {
        Self {
            exec_when_done: false,
            live_on_self: true,
            build_on_save: true,
            hub_server: HubServerConfig::Offline,
            builders: HashMap::new(),
            sync: HashMap::new(),
            builds: vec![],
        }
    }
}

impl MakepadSettings {
    pub fn initial() -> Self {
        Self {
            exec_when_done: false,
            build_on_save: true,
            live_on_self: true,
            hub_server: HubServerConfig::Offline,
            builders: {
                let mut cfg = HashMap::new();
                cfg.insert(
                    "main".to_string(),
                    HubBuilderConfig {
                        http_server: HttpServerConfig::Localhost(8000),
                        workspaces: {
                            let mut workspace = HashMap::new();
                            workspace.insert("wrflib".to_string(), ".".to_string());
                            workspace
                        },
                    },
                );
                cfg
            },
            sync: {
                //sync.insert("main/wrflib".to_string(), vec!["windows/wrflib".to_string()]);
                HashMap::new()
            },
            builds: vec![BuildTarget {
                builder: "main".to_string(),
                workspace: "wrflib".to_string(),
                package: "test_many_quads".to_string(),
                config: "release".to_string(),
            }],
        }
    }
}

#[derive(Clone, Debug, SerRon, DeRon, PartialEq)]
pub struct BuildTarget {
    pub builder: String,
    pub workspace: String,
    pub package: String,
    pub config: String,
}

pub struct MakepadStorage {
    builders_request_uid: HubUid,
    websocket_channels: WebSocketChannels,
    hub_router: Option<HubRouter>,
    hub_server: Option<HubServer>,
    builder_route_send: Option<HubRouteSend>,
    pub hub_ui: Option<HubUI>,
    pub hub_ui_message: Signal,
    pub settings_changed: Signal,
    pub settings_old: MakepadSettings,
    pub settings: MakepadSettings,
    text_buffer_path_to_id: HashMap<String, MakepadTextBufferId>,
    pub text_buffer_id_to_path: HashMap<MakepadTextBufferId, String>,
    pub text_buffers: Vec<MakepadTextBuffer>,
}

pub struct MakepadTextBuffer {
    read_msg: Option<ToHubMsg>,
    pub full_path: String,
    pub text_buffer: TextBuffer,
    pub text_buffer_id: MakepadTextBufferId,
}

#[derive(Clone, Copy, Default, PartialEq, Ord, PartialOrd, Hash, Eq)]
pub struct MakepadTextBufferId(pub usize); //(u16);
impl MakepadTextBufferId {
    pub fn as_index(&self) -> usize {
        self.0
    }
}

const STATUS_NEW_MESSAGE: StatusId = location_hash!();
const STATUS_SETTINGS_CHANGED: StatusId = location_hash!();

impl MakepadStorage {
    pub fn new(cx: &mut Cx) -> Self {
        MakepadStorage {
            builders_request_uid: HubUid::zero(),
            builder_route_send: None,
            websocket_channels: WebSocketChannels::default(),
            hub_router: None,
            hub_server: None,
            hub_ui: None,
            hub_ui_message: cx.new_signal(),
            settings_changed: cx.new_signal(),
            settings_old: MakepadSettings::default(),
            settings: MakepadSettings::default(),
            text_buffer_path_to_id: HashMap::new(),
            text_buffer_id_to_path: HashMap::new(),
            text_buffers: Vec::new(),
        }
    }

    pub fn init(&mut self, cx: &mut Cx) {
        if cx.platform_type.is_desktop() {
            if let Ok(utf8_data) = universal_file::read_to_string("bigedit_settings.ron") {
                self.load_settings(cx, &utf8_data);
            } else {
                // create default settings file
                let def_settings = MakepadSettings::initial();
                let ron = def_settings.serialize_ron();
                cx.file_write("bigedit_settings.ron", ron.as_bytes());
                self.load_settings(cx, &ron);
            }

            // lets start the router
            let mut hub_router = HubRouter::start_hub_router(HubLog::None);
            // lets start the hub UI connected directly
            let hub_ui = HubUI::start_hub_ui_direct(&mut hub_router, {
                let signal = self.hub_ui_message;
                move || {
                    Cx::post_signal(signal, STATUS_NEW_MESSAGE);
                }
            });

            let send = HubBuilder::run_builder_direct("main", self.websocket_channels.clone(), &mut hub_router, |ws, htc| {
                builder::builder(ws, htc)
            });
            self.builder_route_send = Some(send);
            self.hub_router = Some(hub_router);
            self.hub_ui = Some(hub_ui);
        }
    }

    pub fn load_settings(&mut self, cx: &mut Cx, utf8_data: &str) {
        match DeRon::deserialize_ron(utf8_data) {
            Ok(settings) => {
                self.settings_old = self.settings.clone();
                self.settings = settings;
                //self.settings.style_options.scale = self.settings.style_options.scale.min(3.0).max(0.3);
                cx.send_signal(self.settings_changed, STATUS_SETTINGS_CHANGED);

                // so now, here we restart our hub_server if need be.
                if cx.platform_type.is_desktop() && self.settings_old.hub_server != self.settings.hub_server {
                    self.restart_hub_server();
                }
            }
            Err(e) => {
                println!("Cannot deserialize settings {:?}", e);
            }
        }
    }

    fn restart_hub_server(&mut self) {
        if let Some(hub_server) = &mut self.hub_server {
            hub_server.terminate();
        }

        if let Some(hub_router) = &mut self.hub_router {
            let digest = Self::read_or_generate_key_ron();
            // start the server
            self.hub_server = HubServer::start_hub_server(digest, &self.settings.hub_server, hub_router);
        }
    }

    fn read_or_generate_key_ron() -> Digest {
        // read or generate key.ron
        if let Ok(utf8_data) = std::fs::read_to_string("key.ron") {
            if let Ok(digest) = DeRon::deserialize_ron(&utf8_data) {
                return digest;
            }
        }
        let digest = Digest::generate();
        let utf8_data = digest.serialize_ron();
        if std::fs::write("key.ron", utf8_data.as_bytes()).is_err() {
            println!("Cannot generate key.ron");
        }
        digest
    }

    pub fn remap_sync_path(&self, path: &str) -> String {
        let mut path = path.to_string();
        for (key, sync_to) in &self.settings.sync {
            for sync in sync_to {
                if path.starts_with(sync) {
                    path.replace_range(0..sync.len(), key);
                    break;
                }
            }
        }
        path
    }

    fn file_path_to_live_path(fp: &str) -> String {
        if fp.starts_with("main/wrflib/") {
            fp["main/wrflib/".len()..].to_string()
        } else {
            fp.to_string()
        }
    }

    pub fn text_buffer_from_path(&mut self, cx: &mut Cx, path: &str) -> &mut MakepadTextBuffer {
        if let Some(tb_id) = self.text_buffer_path_to_id.get(path) {
            &mut self.text_buffers[tb_id.as_index()]
        } else {
            let tb_id = MakepadTextBufferId(self.text_buffers.len());
            self.text_buffer_path_to_id.insert(path.to_string(), tb_id);
            self.text_buffer_id_to_path.insert(tb_id, path.to_string());
            let mut text_buffer = TextBuffer { signal: cx.new_signal(), ..TextBuffer::default() };
            let live_path = Self::file_path_to_live_path(path);
            log!("live_path {}", &live_path);
            if let Ok(utf8_data) = wrflib::universal_file::read_to_string(&live_path) {
                text_buffer.load_from_utf8(&utf8_data);
            } else {
                text_buffer.load_from_utf8("");
            }
            self.text_buffers.push(MakepadTextBuffer {
                read_msg: None,
                full_path: path.to_string(),
                text_buffer_id: tb_id,
                text_buffer,
            });
            &mut self.text_buffers[tb_id.as_index()]
        }
    }

    pub fn text_buffer_file_write(&mut self, cx: &mut Cx, path: &str) {
        if cx.platform_type.is_desktop() {
            if path.find('/').is_some() {
                if let Some(tb_id) = self.text_buffer_path_to_id.get(path) {
                    let atb = &self.text_buffers[tb_id.as_index()];
                    let hub_ui = self.hub_ui.as_mut().unwrap();
                    let utf8_data = atb.text_buffer.get_as_string();
                    fn send_file_write_request(hub_ui: &HubUI, uid: HubUid, path: &str, data: &Vec<u8>) {
                        if let Some(builder_pos) = path.find('/') {
                            let (builder, rest) = path.split_at(builder_pos);
                            let (_, rest) = rest.split_at(1);

                            hub_ui.route_send.send(ToHubMsg {
                                to: HubMsgTo::Builder(builder.to_string()),
                                msg: HubMsg::FileWriteRequest { uid, path: rest.to_string(), data: data.clone() },
                            });
                        }
                    }
                    // lets write it as a message
                    let uid = hub_ui.route_send.alloc_uid();
                    let utf8_bytes = utf8_data.into_bytes();
                    send_file_write_request(hub_ui, uid, path, &utf8_bytes);
                    // lets send our file write to all sync points.
                    for (sync, points) in &self.settings.sync {
                        if path.starts_with(sync) {
                            for point in points {
                                let mut sync_path = path.to_string();
                                sync_path.replace_range(0..sync.len(), point);
                                send_file_write_request(hub_ui, uid, &sync_path, &utf8_bytes);
                            }
                        }
                    }
                }
            } else {
                // its not a workspace, its a system (settings) file
                if let Some(tb_id) = self.text_buffer_path_to_id.get(path) {
                    let atb = &self.text_buffers[tb_id.as_index()];
                    let utf8_data = atb.text_buffer.get_as_string();
                    cx.file_write(path, utf8_data.as_bytes());
                    // if its the settings, load it
                    if path == "bigedit_settings.ron" {
                        self.load_settings(cx, &utf8_data);
                    };
                }
            }
        }
    }

    pub fn reload_builders(&mut self) {
        let hub_ui = self.hub_ui.as_mut().unwrap();
        let uid = hub_ui.route_send.alloc_uid();
        hub_ui.route_send.send(ToHubMsg { to: HubMsgTo::Hub, msg: HubMsg::ListBuildersRequest { uid } });
        self.builders_request_uid = uid;
    }

    pub fn handle_hub_msg(&mut self, cx: &mut Cx, htc: &FromHubMsg, makepad_windows: &mut Vec<WrfWindow>) {
        let hub_ui = self.hub_ui.as_mut().unwrap();
        // only in ConnectUI of ourselves do we list the workspaces
        match &htc.msg {
            // our own connectUI message, means we are ready to talk to the hub
            HubMsg::ConnectUI => {
                if hub_ui.route_send.is_own_addr(&htc.from) {
                    // now start talking
                }
            }
            HubMsg::DisconnectBuilder(_) | HubMsg::ConnectBuilder(_) => {
                let own = if let Some(send) = &self.builder_route_send { send.is_own_addr(&htc.from) } else { false };
                if !own {
                    self.reload_builders();
                }
            }
            HubMsg::ListBuildersResponse { uid, builders } => {
                if *uid == self.builders_request_uid {
                    let uid = hub_ui.route_send.alloc_uid();
                    // from these workspaces query filetrees
                    for builder in builders {
                        // lets look up a workspace and configure it!
                        // lets config it
                        if let Some(builder_config) = self.settings.builders.get(builder) {
                            hub_ui.route_send.send(ToHubMsg {
                                to: HubMsgTo::Builder(builder.clone()),
                                msg: HubMsg::BuilderConfig { uid, config: builder_config.clone() },
                            });
                        }
                        hub_ui.route_send.send(ToHubMsg {
                            to: HubMsgTo::Builder(builder.clone()),
                            msg: HubMsg::BuilderFileTreeRequest { uid, create_digest: false },
                        });
                        hub_ui
                            .route_send
                            .send(ToHubMsg { to: HubMsgTo::Builder(builder.clone()), msg: HubMsg::ListPackagesRequest { uid } });
                    }
                    self.builders_request_uid = uid;
                    // add all workspace nodes
                    for window in makepad_windows {
                        window.file_panel.file_tree.root_node = FileNode::Folder {
                            name: "".to_string(),
                            draw: None,
                            state: NodeState::Open,
                            folder: builders
                                .iter()
                                .map(|v| FileNode::Folder {
                                    name: v.clone(),
                                    draw: None,
                                    state: NodeState::Open,
                                    folder: Vec::new(),
                                })
                                .chain(std::iter::once(FileNode::File { name: "bigedit_settings.ron".to_string(), draw: None }))
                                .collect(),
                        };
                        cx.request_draw();
                    }
                    // lets resend the file load we haven't gotten
                    for atb in &mut self.text_buffers {
                        if let Some(cth_msg) = &atb.read_msg {
                            hub_ui.route_send.send(cth_msg.clone())
                        }
                    }
                }
            }
            HubMsg::BuilderFileTreeResponse { uid, tree } => {
                if *uid == self.builders_request_uid {
                    // replace a workspace node
                    if let BuilderFileTreeNode::Folder { name, .. } = &tree {
                        let workspace = name.clone();
                        // insert each filetree at the right childnode
                        for window in makepad_windows.iter_mut() {
                            let mut paths = Vec::new();
                            if let FileNode::Folder { folder, .. } = &mut window.file_panel.file_tree.root_node {
                                for node in folder.iter_mut() {
                                    if let FileNode::Folder { name, .. } = node {
                                        if *name == workspace {
                                            *node = hub_to_tree(tree, "", &mut paths);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn hub_to_tree(node: &BuilderFileTreeNode, base: &str, paths: &mut Vec<String>) -> FileNode {
    match node {
        BuilderFileTreeNode::File { name, .. } => {
            let path = format!("{}/{}", base, name);
            paths.push(path);
            FileNode::File { name: name.clone(), draw: None }
        }
        BuilderFileTreeNode::Folder { name, folder, .. } => {
            let path = format!("{}/{}", base, name);
            FileNode::Folder {
                name: name.clone(),
                folder: folder.iter().map(|v| hub_to_tree(v, if base.is_empty() { name } else { &path }, paths)).collect(),
                draw: None,
                state: NodeState::Closed,
            }
        }
    }
}
