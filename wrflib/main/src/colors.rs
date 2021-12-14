// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::cx::*;

pub const COLOR_ALICEBLUE: Vec4 = vec4(0.941_176_5, 0.972_549, 1.0, 1.0);
pub const COLOR_ANTIQUEWHITE: Vec4 = vec4(0.980_392_16, 0.921_568_63, 0.843_137_26, 1.0);
pub const COLOR_AQUA: Vec4 = vec4(0.0, 1.0, 1.0, 1.0);
pub const COLOR_AQUAMARINE: Vec4 = vec4(0.498_039_22, 1.0, 0.831_372_56, 1.0);
pub const COLOR_AZURE: Vec4 = vec4(0.941_176_5, 1.0, 1.0, 1.0);
pub const COLOR_BEIGE: Vec4 = vec4(0.960_784_3, 0.960_784_3, 0.862_745_1, 1.0);
pub const COLOR_BISQUE: Vec4 = vec4(1.0, 0.894_117_65, 0.768_627_46, 1.0);
pub const COLOR_BLACK: Vec4 = vec4(0.0, 0.0, 0.0, 1.0);
pub const COLOR_BLANCHEDALMOND: Vec4 = vec4(1.0, 0.921_568_63, 0.803_921_6, 1.0);
pub const COLOR_BLUE: Vec4 = vec4(0.0, 0.0, 1.0, 1.0);
pub const COLOR_BLUEVIOLET: Vec4 = vec4(0.541_176_5, 0.168_627_46, 0.886_274_5, 1.0);
pub const COLOR_BROWN: Vec4 = vec4(0.647_058_84, 0.164_705_89, 0.164_705_89, 1.0);
pub const COLOR_BURLYWOOD: Vec4 = vec4(0.870_588_24, 0.721_568_64, 0.529_411_8, 1.0);
pub const COLOR_CADETBLUE: Vec4 = vec4(0.372_549_03, 0.619_607_87, 0.627_451, 1.0);
pub const COLOR_CHARTREUSE: Vec4 = vec4(0.498_039_22, 1.0, 0.0, 1.0);
pub const COLOR_CHOCOLATE: Vec4 = vec4(0.823_529_4, 0.411_764_7, 0.117_647_06, 1.0);
pub const COLOR_CORAL: Vec4 = vec4(1.0, 0.498_039_22, 0.313_725_5, 1.0);
pub const COLOR_CORNFLOWER: Vec4 = vec4(0.392_156_87, 0.584_313_75, 0.929_411_77, 1.0);
pub const COLOR_CORNSILK: Vec4 = vec4(1.0, 0.972_549, 0.862_745_1, 1.0);
pub const COLOR_CRIMSON: Vec4 = vec4(0.862_745_1, 0.078_431_375, 0.235_294_12, 1.0);
pub const COLOR_CYAN: Vec4 = vec4(0.0, 1.0, 1.0, 1.0);
pub const COLOR_DARKBLUE: Vec4 = vec4(0.0, 0.0, 0.545_098_07, 1.0);
pub const COLOR_DARKCYAN: Vec4 = vec4(0.0, 0.545_098_07, 0.545_098_07, 1.0);
pub const COLOR_DARKGOLDENROD: Vec4 = vec4(0.721_568_64, 0.525_490_2, 0.043_137_256, 1.0);
pub const COLOR_DARKGRAY: Vec4 = vec4(0.662_745_1, 0.662_745_1, 0.662_745_1, 1.0);
pub const COLOR_DARKGREEN: Vec4 = vec4(0.0, 0.392_156_87, 0.0, 1.0);
pub const COLOR_DARKKHAKI: Vec4 = vec4(0.741_176_5, 0.717_647_1, 0.419_607_85, 1.0);
pub const COLOR_DARKMAGENTA: Vec4 = vec4(0.545_098_07, 0.0, 0.545_098_07, 1.0);
pub const COLOR_DARKOLIVEGREEN: Vec4 = vec4(0.333_333_34, 0.419_607_85, 0.184_313_73, 1.0);
pub const COLOR_DARKORANGE: Vec4 = vec4(1.0, 0.549_019_63, 0.0, 1.0);
pub const COLOR_DARKORCHID: Vec4 = vec4(0.6, 0.196_078_43, 0.8, 1.0);
pub const COLOR_DARKRED: Vec4 = vec4(0.545_098_07, 0.0, 0.0, 1.0);
pub const COLOR_DARKSALMON: Vec4 = vec4(0.913_725_5, 0.588_235_3, 0.478_431_37, 1.0);
pub const COLOR_DARKSEAGREEN: Vec4 = vec4(0.560_784_34, 0.737_254_9, 0.560_784_34, 1.0);
pub const COLOR_DARKSLATEBLUE: Vec4 = vec4(0.282_352_95, 0.239_215_69, 0.545_098_07, 1.0);
pub const COLOR_DARKSLATEGRAY: Vec4 = vec4(0.184_313_73, 0.309_803_93, 0.309_803_93, 1.0);
pub const COLOR_DARKTURQUOISE: Vec4 = vec4(0.0, 0.807_843_15, 0.819_607_85, 1.0);
pub const COLOR_DARKVIOLET: Vec4 = vec4(0.580_392_2, 0.0, 0.827_451, 1.0);
pub const COLOR_DEEPPINK: Vec4 = vec4(1.0, 0.078_431_375, 0.576_470_6, 1.0);
pub const COLOR_DEEPSKYBLUE: Vec4 = vec4(0.0, 0.749_019_6, 1.0, 1.0);
pub const COLOR_DIMGRAY: Vec4 = vec4(0.411_764_7, 0.411_764_7, 0.411_764_7, 1.0);
pub const COLOR_DODGERBLUE: Vec4 = vec4(0.117_647_06, 0.564_705_9, 1.0, 1.0);
pub const COLOR_FIREBRICK: Vec4 = vec4(0.698_039_23, 0.133_333_34, 0.133_333_34, 1.0);
pub const COLOR_FLORALWHITE: Vec4 = vec4(1.0, 0.980_392_16, 0.941_176_5, 1.0);
pub const COLOR_FORESTGREEN: Vec4 = vec4(0.133_333_34, 0.545_098_07, 0.133_333_34, 1.0);
pub const COLOR_FUCHSIA: Vec4 = vec4(1.0, 0.0, 1.0, 1.0);
pub const COLOR_GAINSBORO: Vec4 = vec4(0.862_745_1, 0.862_745_1, 0.862_745_1, 1.0);
pub const COLOR_GHOSTWHITE: Vec4 = vec4(0.972_549, 0.972_549, 1.0, 1.0);
pub const COLOR_GOLD: Vec4 = vec4(1.0, 0.843_137_26, 0.0, 1.0);
pub const COLOR_GOLDENROD: Vec4 = vec4(0.854_901_97, 0.647_058_84, 0.125_490_2, 1.0);
pub const COLOR_GRAY: Vec4 = vec4(0.745_098_05, 0.745_098_05, 0.745_098_05, 1.0);
pub const COLOR_WEBGRAY: Vec4 = vec4(0.501_960_8, 0.501_960_8, 0.501_960_8, 1.0);
pub const COLOR_GREEN: Vec4 = vec4(0.0, 1.0, 0.0, 1.0);
pub const COLOR_WEBGREEN: Vec4 = vec4(0.0, 0.501_960_8, 0.0, 1.0);
pub const COLOR_GREENYELLOW: Vec4 = vec4(0.678_431_4, 1.0, 0.184_313_73, 1.0);
pub const COLOR_HONEYDEW: Vec4 = vec4(0.941_176_5, 1.0, 0.941_176_5, 1.0);
pub const COLOR_HOTPINK: Vec4 = vec4(1.0, 0.411_764_7, 0.705_882_4, 1.0);
pub const COLOR_INDIANRED: Vec4 = vec4(0.803_921_6, 0.360_784_32, 0.360_784_32, 1.0);
pub const COLOR_INDIGO: Vec4 = vec4(0.294_117_66, 0.0, 0.509_803_95, 1.0);
pub const COLOR_IVORY: Vec4 = vec4(1.0, 1.0, 0.941_176_5, 1.0);
pub const COLOR_KHAKI: Vec4 = vec4(0.941_176_5, 0.901_960_8, 0.549_019_63, 1.0);
pub const COLOR_LAVENDER: Vec4 = vec4(0.901_960_8, 0.901_960_8, 0.980_392_16, 1.0);
pub const COLOR_LAVENDERBLUSH: Vec4 = vec4(1.0, 0.941_176_5, 0.960_784_3, 1.0);
pub const COLOR_LAWNGREEN: Vec4 = vec4(0.486_274_5, 0.988_235_3, 0.0, 1.0);
pub const COLOR_LEMONCHIFFON: Vec4 = vec4(1.0, 0.980_392_16, 0.803_921_6, 1.0);
pub const COLOR_LIGHTBLUE: Vec4 = vec4(0.678_431_4, 0.847_058_83, 0.901_960_8, 1.0);
pub const COLOR_LIGHTCORAL: Vec4 = vec4(0.941_176_5, 0.501_960_8, 0.501_960_8, 1.0);
pub const COLOR_LIGHTCYAN: Vec4 = vec4(0.878_431_4, 1.0, 1.0, 1.0);
pub const COLOR_LIGHTGOLDENROD: Vec4 = vec4(0.980_392_16, 0.980_392_16, 0.823_529_4, 1.0);
pub const COLOR_LIGHTGRAY: Vec4 = vec4(0.827_451, 0.827_451, 0.827_451, 1.0);
pub const COLOR_LIGHTGREEN: Vec4 = vec4(0.564_705_9, 0.933_333_34, 0.564_705_9, 1.0);
pub const COLOR_LIGHTPINK: Vec4 = vec4(1.0, 0.713_725_5, 0.756_862_76, 1.0);
pub const COLOR_LIGHTSALMON: Vec4 = vec4(1.0, 0.627_451, 0.478_431_37, 1.0);
pub const COLOR_LIGHTSEAGREEN: Vec4 = vec4(0.125_490_2, 0.698_039_23, 0.666_666_7, 1.0);
pub const COLOR_LIGHTSKYBLUE: Vec4 = vec4(0.529_411_8, 0.807_843_15, 0.980_392_16, 1.0);
pub const COLOR_LIGHTSLATEGRAY: Vec4 = vec4(0.466_666_67, 0.533_333_36, 0.6, 1.0);
pub const COLOR_LIGHTSTEELBLUE: Vec4 = vec4(0.690_196_1, 0.768_627_46, 0.870_588_24, 1.0);
pub const COLOR_LIGHTYELLOW: Vec4 = vec4(1.0, 1.0, 0.878_431_4, 1.0);
pub const COLOR_LIME: Vec4 = vec4(0.0, 1.0, 0.0, 1.0);
pub const COLOR_LIMEGREEN: Vec4 = vec4(0.196_078_43, 0.803_921_6, 0.196_078_43, 1.0);
pub const COLOR_LINEN: Vec4 = vec4(0.980_392_16, 0.941_176_5, 0.901_960_8, 1.0);
pub const COLOR_MAGENTA: Vec4 = vec4(1.0, 0.0, 1.0, 1.0);
pub const COLOR_MAROON: Vec4 = vec4(0.690_196_1, 0.188_235_3, 0.376_470_6, 1.0);
pub const COLOR_WEBMAROON: Vec4 = vec4(0.498_039_22, 0.0, 0.0, 1.0);
pub const COLOR_MEDIUMAQUAMARINE: Vec4 = vec4(0.4, 0.803_921_6, 0.666_666_7, 1.0);
pub const COLOR_MEDIUMBLUE: Vec4 = vec4(0.0, 0.0, 0.803_921_6, 1.0);
pub const COLOR_MEDIUMORCHID: Vec4 = vec4(0.729_411_8, 0.333_333_34, 0.827_451, 1.0);
pub const COLOR_MEDIUMPURPLE: Vec4 = vec4(0.576_470_6, 0.439_215_7, 0.858_823_54, 1.0);
pub const COLOR_MEDIUMSEAGREEN: Vec4 = vec4(0.235_294_12, 0.701_960_8, 0.443_137_26, 1.0);
pub const COLOR_MEDIUMSLATEBLUE: Vec4 = vec4(0.482_352_94, 0.407_843_14, 0.933_333_34, 1.0);
pub const COLOR_MEDIUMSPRINGGREEN: Vec4 = vec4(0.0, 0.980_392_16, 0.603_921_6, 1.0);
pub const COLOR_MEDIUMTURQUOISE: Vec4 = vec4(0.282_352_95, 0.819_607_85, 0.8, 1.0);
pub const COLOR_MEDIUMVIOLETRED: Vec4 = vec4(0.780_392_17, 0.082_352_94, 0.521_568_66, 1.0);
pub const COLOR_MIDNIGHTBLUE: Vec4 = vec4(0.098_039_22, 0.098_039_22, 0.439_215_7, 1.0);
pub const COLOR_MINTCREAM: Vec4 = vec4(0.960_784_3, 1.0, 0.980_392_16, 1.0);
pub const COLOR_MISTYROSE: Vec4 = vec4(1.0, 0.894_117_65, 0.882_352_95, 1.0);
pub const COLOR_MOCCASIN: Vec4 = vec4(1.0, 0.894_117_65, 0.709_803_94, 1.0);
pub const COLOR_NAVAJOWHITE: Vec4 = vec4(1.0, 0.870_588_24, 0.678_431_4, 1.0);
pub const COLOR_NAVYBLUE: Vec4 = vec4(0.0, 0.0, 0.501_960_8, 1.0);
pub const COLOR_OLDLACE: Vec4 = vec4(0.992_156_86, 0.960_784_3, 0.901_960_8, 1.0);
pub const COLOR_OLIVE: Vec4 = vec4(0.501_960_8, 0.501_960_8, 0.0, 1.0);
pub const COLOR_OLIVEDRAB: Vec4 = vec4(0.419_607_85, 0.556_862_8, 0.137_254_91, 1.0);
pub const COLOR_ORANGE: Vec4 = vec4(1.0, 0.647_058_84, 0.0, 1.0);
pub const COLOR_ORANGERED: Vec4 = vec4(1.0, 0.270_588_25, 0.0, 1.0);
pub const COLOR_ORCHID: Vec4 = vec4(0.854_901_97, 0.439_215_7, 0.839_215_7, 1.0);
pub const COLOR_PALEGOLDENROD: Vec4 = vec4(0.933_333_34, 0.909_803_9, 0.666_666_7, 1.0);
pub const COLOR_PALEGREEN: Vec4 = vec4(0.596_078_46, 0.984_313_7, 0.596_078_46, 1.0);
pub const COLOR_PALETURQUOISE: Vec4 = vec4(0.686_274_5, 0.933_333_34, 0.933_333_34, 1.0);
pub const COLOR_PALEVIOLETRED: Vec4 = vec4(0.858_823_54, 0.439_215_7, 0.576_470_6, 1.0);
pub const COLOR_PAPAYAWHIP: Vec4 = vec4(1.0, 0.937_254_9, 0.835_294_1, 1.0);
pub const COLOR_PEACHPUFF: Vec4 = vec4(1.0, 0.854_901_97, 0.725_490_2, 1.0);
pub const COLOR_PERU: Vec4 = vec4(0.803_921_6, 0.521_568_66, 0.247_058_82, 1.0);
pub const COLOR_PINK: Vec4 = vec4(1.0, 0.752_941_2, 0.796_078_44, 1.0);
pub const COLOR_PLUM: Vec4 = vec4(0.866_666_7, 0.627_451, 0.866_666_7, 1.0);
pub const COLOR_POWDERBLUE: Vec4 = vec4(0.690_196_1, 0.878_431_4, 0.901_960_8, 1.0);
pub const COLOR_PURPLE: Vec4 = vec4(0.627_451, 0.125_490_2, 0.941_176_5, 1.0);
pub const COLOR_WEBPURPLE: Vec4 = vec4(0.498_039_22, 0.0, 0.498_039_22, 1.0);
pub const COLOR_REBECCAPURPLE: Vec4 = vec4(0.4, 0.2, 0.6, 1.0);
pub const COLOR_RED: Vec4 = vec4(1.0, 0.0, 0.0, 1.0);
pub const COLOR_ROSYBROWN: Vec4 = vec4(0.737_254_9, 0.560_784_34, 0.560_784_34, 1.0);
pub const COLOR_ROYALBLUE: Vec4 = vec4(0.254_901_98, 0.411_764_7, 0.882_352_95, 1.0);
pub const COLOR_SADDLEBROWN: Vec4 = vec4(0.545_098_07, 0.270_588_25, 0.074_509_81, 1.0);
pub const COLOR_SALMON: Vec4 = vec4(0.980_392_16, 0.501_960_8, 0.447_058_83, 1.0);
pub const COLOR_SANDYBROWN: Vec4 = vec4(0.956_862_75, 0.643_137_3, 0.376_470_6, 1.0);
pub const COLOR_SEAGREEN: Vec4 = vec4(0.180_392_16, 0.545_098_07, 0.341_176_48, 1.0);
pub const COLOR_SEASHELL: Vec4 = vec4(1.0, 0.960_784_3, 0.933_333_34, 1.0);
pub const COLOR_SIENNA: Vec4 = vec4(0.627_451, 0.321_568_64, 0.176_470_6, 1.0);
pub const COLOR_SILVER: Vec4 = vec4(0.752_941_2, 0.752_941_2, 0.752_941_2, 1.0);
pub const COLOR_SKYBLUE: Vec4 = vec4(0.529_411_8, 0.807_843_15, 0.921_568_63, 1.0);
pub const COLOR_SLATEBLUE: Vec4 = vec4(0.415_686_28, 0.352_941_2, 0.803_921_6, 1.0);
pub const COLOR_SLATEGRAY: Vec4 = vec4(0.439_215_7, 0.501_960_8, 0.564_705_9, 1.0);
pub const COLOR_SNOW: Vec4 = vec4(1.0, 0.980_392_16, 0.980_392_16, 1.0);
pub const COLOR_SPRINGGREEN: Vec4 = vec4(0.0, 1.0, 0.498_039_22, 1.0);
pub const COLOR_STEELBLUE: Vec4 = vec4(0.274_509_82, 0.509_803_95, 0.705_882_4, 1.0);
pub const COLOR_TAN: Vec4 = vec4(0.823_529_4, 0.705_882_4, 0.549_019_63, 1.0);
pub const COLOR_TEAL: Vec4 = vec4(0.0, 0.501_960_8, 0.501_960_8, 1.0);
pub const COLOR_THISTLE: Vec4 = vec4(0.847_058_83, 0.749_019_6, 0.847_058_83, 1.0);
pub const COLOR_TOMATO: Vec4 = vec4(1.0, 0.388_235_3, 0.278_431_4, 1.0);
pub const COLOR_TURQUOISE: Vec4 = vec4(0.250_980_4, 0.878_431_4, 0.815_686_3, 1.0);
pub const COLOR_VIOLET: Vec4 = vec4(0.933_333_34, 0.509_803_95, 0.933_333_34, 1.0);
pub const COLOR_WHEAT: Vec4 = vec4(0.960_784_3, 0.870_588_24, 0.701_960_8, 1.0);
pub const COLOR_WHITE: Vec4 = vec4(1.0, 1.0, 1.0, 1.0);
pub const COLOR_WHITESMOKE: Vec4 = vec4(0.960_784_3, 0.960_784_3, 0.960_784_3, 1.0);
pub const COLOR_YELLOW: Vec4 = vec4(1.0, 1.0, 0.0, 1.0);
pub const COLOR_YELLOWGREEN: Vec4 = vec4(0.603_921_6, 0.803_921_6, 0.196_078_43, 1.0);
pub const COLOR_RED500: Vec4 = vec4(0.956_862_75, 0.262_745_1, 0.211_764_71, 1.0);
pub const COLOR_RED50: Vec4 = vec4(1.0, 0.921_568_63, 0.933_333_34, 1.0);
pub const COLOR_RED100: Vec4 = vec4(1.0, 0.803_921_6, 0.823_529_4, 1.0);
pub const COLOR_RED200: Vec4 = vec4(0.937_254_9, 0.603_921_6, 0.603_921_6, 1.0);
pub const COLOR_RED300: Vec4 = vec4(0.898_039_2, 0.450_980_4, 0.450_980_4, 1.0);
pub const COLOR_RED400: Vec4 = vec4(0.937_254_9, 0.325_490_2, 0.313_725_5, 1.0);
pub const COLOR_RED600: Vec4 = vec4(0.898_039_2, 0.223_529_41, 0.207_843_14, 1.0);
pub const COLOR_RED700: Vec4 = vec4(0.827_451, 0.184_313_73, 0.184_313_73, 1.0);
pub const COLOR_RED800: Vec4 = vec4(0.776_470_6, 0.156_862_75, 0.156_862_75, 1.0);
pub const COLOR_RED900: Vec4 = vec4(0.717_647_1, 0.109_803_92, 0.109_803_92, 1.0);
pub const COLOR_REDA100: Vec4 = vec4(1.0, 0.541_176_5, 0.501_960_8, 1.0);
pub const COLOR_REDA200: Vec4 = vec4(1.0, 0.321_568_64, 0.321_568_64, 1.0);
pub const COLOR_REDA400: Vec4 = vec4(1.0, 0.090_196_08, 0.266_666_68, 1.0);
pub const COLOR_REDA700: Vec4 = vec4(0.835_294_1, 0.0, 0.0, 1.0);
pub const COLOR_PINK500: Vec4 = vec4(0.913_725_5, 0.117_647_06, 0.388_235_3, 1.0);
pub const COLOR_PINK50: Vec4 = vec4(0.988_235_3, 0.894_117_65, 0.925_490_2, 1.0);
pub const COLOR_PINK100: Vec4 = vec4(0.972_549, 0.733_333_35, 0.815_686_3, 1.0);
pub const COLOR_PINK200: Vec4 = vec4(0.956_862_75, 0.560_784_34, 0.694_117_67, 1.0);
pub const COLOR_PINK300: Vec4 = vec4(0.941_176_5, 0.384_313_73, 0.572_549_05, 1.0);
pub const COLOR_PINK400: Vec4 = vec4(0.925_490_2, 0.250_980_4, 0.478_431_37, 1.0);
pub const COLOR_PINK600: Vec4 = vec4(0.847_058_83, 0.105_882_354, 0.376_470_6, 1.0);
pub const COLOR_PINK700: Vec4 = vec4(0.760_784_3, 0.094_117_65, 0.356_862_75, 1.0);
pub const COLOR_PINK800: Vec4 = vec4(0.678_431_4, 0.078_431_375, 0.341_176_48, 1.0);
pub const COLOR_PINK900: Vec4 = vec4(0.533_333_36, 0.054_901_96, 0.309_803_93, 1.0);
pub const COLOR_PINKA100: Vec4 = vec4(1.0, 0.501_960_8, 0.670_588_25, 1.0);
pub const COLOR_PINKA200: Vec4 = vec4(1.0, 0.250_980_4, 0.505_882_4, 1.0);
pub const COLOR_PINKA400: Vec4 = vec4(0.960_784_3, 0.0, 0.341_176_48, 1.0);
pub const COLOR_PINKA700: Vec4 = vec4(0.772_549_03, 0.066_666_67, 0.384_313_73, 1.0);
pub const COLOR_PURPLE500: Vec4 = vec4(0.611_764_7, 0.152_941_18, 0.690_196_1, 1.0);
pub const COLOR_PURPLE50: Vec4 = vec4(0.952_941_2, 0.898_039_2, 0.960_784_3, 1.0);
pub const COLOR_PURPLE100: Vec4 = vec4(0.882_352_95, 0.745_098_05, 0.905_882_36, 1.0);
pub const COLOR_PURPLE200: Vec4 = vec4(0.807_843_15, 0.576_470_6, 0.847_058_83, 1.0);
pub const COLOR_PURPLE300: Vec4 = vec4(0.729_411_8, 0.407_843_14, 0.784_313_74, 1.0);
pub const COLOR_PURPLE400: Vec4 = vec4(0.670_588_25, 0.278_431_4, 0.737_254_9, 1.0);
pub const COLOR_PURPLE600: Vec4 = vec4(0.556_862_8, 0.141_176_48, 0.666_666_7, 1.0);
pub const COLOR_PURPLE700: Vec4 = vec4(0.482_352_94, 0.121_568_63, 0.635_294_14, 1.0);
pub const COLOR_PURPLE800: Vec4 = vec4(0.415_686_28, 0.105_882_354, 0.603_921_6, 1.0);
pub const COLOR_PURPLE900: Vec4 = vec4(0.290_196_1, 0.078_431_375, 0.549_019_63, 1.0);
pub const COLOR_PURPLEA100: Vec4 = vec4(0.917_647_06, 0.501_960_8, 0.988_235_3, 1.0);
pub const COLOR_PURPLEA200: Vec4 = vec4(0.878_431_4, 0.250_980_4, 0.984_313_7, 1.0);
pub const COLOR_PURPLEA400: Vec4 = vec4(0.835_294_1, 0.0, 0.976_470_6, 1.0);
pub const COLOR_PURPLEA700: Vec4 = vec4(0.666_666_7, 0.0, 1.0, 1.0);
pub const COLOR_DEEPPURPLE500: Vec4 = vec4(0.403_921_57, 0.227_450_98, 0.717_647_1, 1.0);
pub const COLOR_DEEPPURPLE50: Vec4 = vec4(0.929_411_77, 0.905_882_36, 0.964_705_9, 1.0);
pub const COLOR_DEEPPURPLE100: Vec4 = vec4(0.819_607_85, 0.768_627_46, 0.913_725_5, 1.0);
pub const COLOR_DEEPPURPLE200: Vec4 = vec4(0.701_960_8, 0.615_686_3, 0.858_823_54, 1.0);
pub const COLOR_DEEPPURPLE300: Vec4 = vec4(0.584_313_75, 0.458_823_53, 0.803_921_6, 1.0);
pub const COLOR_DEEPPURPLE400: Vec4 = vec4(0.494_117_65, 0.341_176_48, 0.760_784_3, 1.0);
pub const COLOR_DEEPPURPLE600: Vec4 = vec4(0.368_627_46, 0.207_843_14, 0.694_117_67, 1.0);
pub const COLOR_DEEPPURPLE700: Vec4 = vec4(0.317_647_07, 0.176_470_6, 0.658_823_55, 1.0);
pub const COLOR_DEEPPURPLE800: Vec4 = vec4(0.270_588_25, 0.152_941_18, 0.627_451, 1.0);
pub const COLOR_DEEPPURPLE900: Vec4 = vec4(0.192_156_87, 0.105_882_354, 0.572_549_05, 1.0);
pub const COLOR_DEEPPURPLEA100: Vec4 = vec4(0.701_960_8, 0.533_333_36, 1.0, 1.0);
pub const COLOR_DEEPPURPLEA200: Vec4 = vec4(0.486_274_5, 0.301_960_8, 1.0, 1.0);
pub const COLOR_DEEPPURPLEA400: Vec4 = vec4(0.396_078_44, 0.121_568_63, 1.0, 1.0);
pub const COLOR_DEEPPURPLEA700: Vec4 = vec4(0.384_313_73, 0.0, 0.917_647_06, 1.0);
pub const COLOR_INDIGO500: Vec4 = vec4(0.247_058_82, 0.317_647_07, 0.709_803_94, 1.0);
pub const COLOR_INDIGO50: Vec4 = vec4(0.909_803_9, 0.917_647_06, 0.964_705_9, 1.0);
pub const COLOR_INDIGO100: Vec4 = vec4(0.772_549_03, 0.792_156_9, 0.913_725_5, 1.0);
pub const COLOR_INDIGO200: Vec4 = vec4(0.623_529_43, 0.658_823_55, 0.854_901_97, 1.0);
pub const COLOR_INDIGO300: Vec4 = vec4(0.474_509_8, 0.525_490_2, 0.796_078_44, 1.0);
pub const COLOR_INDIGO400: Vec4 = vec4(0.360_784_32, 0.419_607_85, 0.752_941_2, 1.0);
pub const COLOR_INDIGO600: Vec4 = vec4(0.223_529_41, 0.286_274_52, 0.670_588_25, 1.0);
pub const COLOR_INDIGO700: Vec4 = vec4(0.188_235_3, 0.247_058_82, 0.623_529_43, 1.0);
pub const COLOR_INDIGO800: Vec4 = vec4(0.156_862_75, 0.207_843_14, 0.576_470_6, 1.0);
pub const COLOR_INDIGO900: Vec4 = vec4(0.101_960_786, 0.137_254_91, 0.494_117_65, 1.0);
pub const COLOR_INDIGOA100: Vec4 = vec4(0.549_019_63, 0.619_607_87, 1.0, 1.0);
pub const COLOR_INDIGOA200: Vec4 = vec4(0.325_490_2, 0.427_450_98, 0.996_078_43, 1.0);
pub const COLOR_INDIGOA400: Vec4 = vec4(0.239_215_69, 0.352_941_2, 0.996_078_43, 1.0);
pub const COLOR_INDIGOA700: Vec4 = vec4(0.188_235_3, 0.309_803_93, 0.996_078_43, 1.0);
pub const COLOR_BLUE500: Vec4 = vec4(0.129_411_77, 0.588_235_3, 0.952_941_2, 1.0);
pub const COLOR_BLUE50: Vec4 = vec4(0.890_196_1, 0.949_019_6, 0.992_156_86, 1.0);
pub const COLOR_BLUE100: Vec4 = vec4(0.733_333_35, 0.870_588_24, 0.984_313_7, 1.0);
pub const COLOR_BLUE200: Vec4 = vec4(0.564_705_9, 0.792_156_9, 0.976_470_6, 1.0);
pub const COLOR_BLUE300: Vec4 = vec4(0.392_156_87, 0.709_803_94, 0.964_705_9, 1.0);
pub const COLOR_BLUE400: Vec4 = vec4(0.258_823_54, 0.647_058_84, 0.960_784_3, 1.0);
pub const COLOR_BLUE600: Vec4 = vec4(0.117_647_06, 0.533_333_36, 0.898_039_2, 1.0);
pub const COLOR_BLUE700: Vec4 = vec4(0.098_039_22, 0.462_745_1, 0.823_529_4, 1.0);
pub const COLOR_BLUE800: Vec4 = vec4(0.082_352_94, 0.396_078_44, 0.752_941_2, 1.0);
pub const COLOR_BLUE900: Vec4 = vec4(0.050_980_393, 0.278_431_4, 0.631_372_6, 1.0);
pub const COLOR_BLUEA100: Vec4 = vec4(0.509_803_95, 0.694_117_67, 1.0, 1.0);
pub const COLOR_BLUEA200: Vec4 = vec4(0.266_666_68, 0.541_176_5, 1.0, 1.0);
pub const COLOR_BLUEA400: Vec4 = vec4(0.160_784_32, 0.474_509_8, 1.0, 1.0);
pub const COLOR_BLUEA700: Vec4 = vec4(0.160_784_32, 0.384_313_73, 1.0, 1.0);
pub const COLOR_LIGHTBLUE500: Vec4 = vec4(0.011_764_706, 0.662_745_1, 0.956_862_75, 1.0);
pub const COLOR_LIGHTBLUE50: Vec4 = vec4(0.882_352_95, 0.960_784_3, 0.996_078_43, 1.0);
pub const COLOR_LIGHTBLUE100: Vec4 = vec4(0.701_960_8, 0.898_039_2, 0.988_235_3, 1.0);
pub const COLOR_LIGHTBLUE200: Vec4 = vec4(0.505_882_4, 0.831_372_56, 0.980_392_16, 1.0);
pub const COLOR_LIGHTBLUE300: Vec4 = vec4(0.309_803_93, 0.764_705_9, 0.968_627_45, 1.0);
pub const COLOR_LIGHTBLUE400: Vec4 = vec4(0.160_784_32, 0.713_725_5, 0.964_705_9, 1.0);
pub const COLOR_LIGHTBLUE600: Vec4 = vec4(0.011_764_706, 0.607_843_16, 0.898_039_2, 1.0);
pub const COLOR_LIGHTBLUE700: Vec4 = vec4(0.007_843_138, 0.533_333_36, 0.819_607_85, 1.0);
pub const COLOR_LIGHTBLUE800: Vec4 = vec4(0.007_843_138, 0.466_666_67, 0.741_176_5, 1.0);
pub const COLOR_LIGHTBLUE900: Vec4 = vec4(0.003_921_569, 0.341_176_48, 0.607_843_16, 1.0);
pub const COLOR_LIGHTBLUEA100: Vec4 = vec4(0.501_960_8, 0.847_058_83, 1.0, 1.0);
pub const COLOR_LIGHTBLUEA200: Vec4 = vec4(0.250_980_4, 0.768_627_46, 1.0, 1.0);
pub const COLOR_LIGHTBLUEA400: Vec4 = vec4(0.0, 0.690_196_1, 1.0, 1.0);
pub const COLOR_LIGHTBLUEA700: Vec4 = vec4(0.0, 0.568_627_5, 0.917_647_06, 1.0);
pub const COLOR_CYAN500: Vec4 = vec4(0.0, 0.737_254_9, 0.831_372_56, 1.0);
pub const COLOR_CYAN50: Vec4 = vec4(0.878_431_4, 0.968_627_45, 0.980_392_16, 1.0);
pub const COLOR_CYAN100: Vec4 = vec4(0.698_039_23, 0.921_568_63, 0.949_019_6, 1.0);
pub const COLOR_CYAN200: Vec4 = vec4(0.501_960_8, 0.870_588_24, 0.917_647_06, 1.0);
pub const COLOR_CYAN300: Vec4 = vec4(0.301_960_8, 0.815_686_3, 0.882_352_95, 1.0);
pub const COLOR_CYAN400: Vec4 = vec4(0.149_019_61, 0.776_470_6, 0.854_901_97, 1.0);
pub const COLOR_CYAN600: Vec4 = vec4(0.0, 0.674_509_8, 0.756_862_76, 1.0);
pub const COLOR_CYAN700: Vec4 = vec4(0.0, 0.592_156_9, 0.654_902, 1.0);
pub const COLOR_CYAN800: Vec4 = vec4(0.0, 0.513_725_5, 0.560_784_34, 1.0);
pub const COLOR_CYAN900: Vec4 = vec4(0.0, 0.376_470_6, 0.392_156_87, 1.0);
pub const COLOR_CYANA100: Vec4 = vec4(0.517_647_1, 1.0, 1.0, 1.0);
pub const COLOR_CYANA200: Vec4 = vec4(0.094_117_65, 1.0, 1.0, 1.0);
pub const COLOR_CYANA400: Vec4 = vec4(0.0, 0.898_039_2, 1.0, 1.0);
pub const COLOR_CYANA700: Vec4 = vec4(0.0, 0.721_568_64, 0.831_372_56, 1.0);
pub const COLOR_TEAL500: Vec4 = vec4(0.0, 0.588_235_3, 0.533_333_36, 1.0);
pub const COLOR_TEAL50: Vec4 = vec4(0.878_431_4, 0.949_019_6, 0.945_098_04, 1.0);
pub const COLOR_TEAL100: Vec4 = vec4(0.698_039_23, 0.874_509_8, 0.858_823_54, 1.0);
pub const COLOR_TEAL200: Vec4 = vec4(0.501_960_8, 0.796_078_44, 0.768_627_46, 1.0);
pub const COLOR_TEAL300: Vec4 = vec4(0.301_960_8, 0.713_725_5, 0.674_509_8, 1.0);
pub const COLOR_TEAL400: Vec4 = vec4(0.149_019_61, 0.650_980_4, 0.603_921_6, 1.0);
pub const COLOR_TEAL600: Vec4 = vec4(0.0, 0.537_254_9, 0.482_352_94, 1.0);
pub const COLOR_TEAL700: Vec4 = vec4(0.0, 0.474_509_8, 0.419_607_85, 1.0);
pub const COLOR_TEAL800: Vec4 = vec4(0.0, 0.411_764_7, 0.360_784_32, 1.0);
pub const COLOR_TEAL900: Vec4 = vec4(0.0, 0.301_960_8, 0.250_980_4, 1.0);
pub const COLOR_TEALA100: Vec4 = vec4(0.654_902, 1.0, 0.921_568_63, 1.0);
pub const COLOR_TEALA200: Vec4 = vec4(0.392_156_87, 1.0, 0.854_901_97, 1.0);
pub const COLOR_TEALA400: Vec4 = vec4(0.113_725_49, 0.913_725_5, 0.713_725_5, 1.0);
pub const COLOR_TEALA700: Vec4 = vec4(0.0, 0.749_019_6, 0.647_058_84, 1.0);
pub const COLOR_GREEN500: Vec4 = vec4(0.298_039_23, 0.686_274_5, 0.313_725_5, 1.0);
pub const COLOR_GREEN50: Vec4 = vec4(0.909_803_9, 0.960_784_3, 0.913_725_5, 1.0);
pub const COLOR_GREEN100: Vec4 = vec4(0.784_313_74, 0.901_960_8, 0.788_235_3, 1.0);
pub const COLOR_GREEN200: Vec4 = vec4(0.647_058_84, 0.839_215_7, 0.654_902, 1.0);
pub const COLOR_GREEN300: Vec4 = vec4(0.505_882_4, 0.780_392_17, 0.517_647_1, 1.0);
pub const COLOR_GREEN400: Vec4 = vec4(0.4, 0.733_333_35, 0.415_686_28, 1.0);
pub const COLOR_GREEN600: Vec4 = vec4(0.262_745_1, 0.627_451, 0.278_431_4, 1.0);
pub const COLOR_GREEN700: Vec4 = vec4(0.219_607_84, 0.556_862_8, 0.235_294_12, 1.0);
pub const COLOR_GREEN800: Vec4 = vec4(0.180_392_16, 0.490_196_08, 0.196_078_43, 1.0);
pub const COLOR_GREEN900: Vec4 = vec4(0.105_882_354, 0.368_627_46, 0.125_490_2, 1.0);
pub const COLOR_GREENA100: Vec4 = vec4(0.725_490_2, 0.964_705_9, 0.792_156_9, 1.0);
pub const COLOR_GREENA200: Vec4 = vec4(0.411_764_7, 0.941_176_5, 0.682_352_96, 1.0);
pub const COLOR_GREENA400: Vec4 = vec4(0.0, 0.901_960_8, 0.462_745_1, 1.0);
pub const COLOR_GREENA700: Vec4 = vec4(0.0, 0.784_313_74, 0.325_490_2, 1.0);
pub const COLOR_LIGHTGREEN500: Vec4 = vec4(0.545_098_07, 0.764_705_9, 0.290_196_1, 1.0);
pub const COLOR_LIGHTGREEN50: Vec4 = vec4(0.945_098_04, 0.972_549, 0.913_725_5, 1.0);
pub const COLOR_LIGHTGREEN100: Vec4 = vec4(0.862_745_1, 0.929_411_77, 0.784_313_74, 1.0);
pub const COLOR_LIGHTGREEN200: Vec4 = vec4(0.772_549_03, 0.882_352_95, 0.647_058_84, 1.0);
pub const COLOR_LIGHTGREEN300: Vec4 = vec4(0.682_352_96, 0.835_294_1, 0.505_882_4, 1.0);
pub const COLOR_LIGHTGREEN400: Vec4 = vec4(0.611_764_7, 0.8, 0.396_078_44, 1.0);
pub const COLOR_LIGHTGREEN600: Vec4 = vec4(0.486_274_5, 0.701_960_8, 0.258_823_54, 1.0);
pub const COLOR_LIGHTGREEN700: Vec4 = vec4(0.407_843_14, 0.623_529_43, 0.219_607_84, 1.0);
pub const COLOR_LIGHTGREEN800: Vec4 = vec4(0.333_333_34, 0.545_098_07, 0.184_313_73, 1.0);
pub const COLOR_LIGHTGREEN900: Vec4 = vec4(0.2, 0.411_764_7, 0.117_647_06, 1.0);
pub const COLOR_LIGHTGREENA100: Vec4 = vec4(0.8, 1.0, 0.564_705_9, 1.0);
pub const COLOR_LIGHTGREENA200: Vec4 = vec4(0.698_039_23, 1.0, 0.349_019_62, 1.0);
pub const COLOR_LIGHTGREENA400: Vec4 = vec4(0.462_745_1, 1.0, 0.011_764_706, 1.0);
pub const COLOR_LIGHTGREENA700: Vec4 = vec4(0.392_156_87, 0.866_666_7, 0.090_196_08, 1.0);
pub const COLOR_LIME500: Vec4 = vec4(0.803_921_6, 0.862_745_1, 0.223_529_41, 1.0);
pub const COLOR_LIME50: Vec4 = vec4(0.976_470_6, 0.984_313_7, 0.905_882_36, 1.0);
pub const COLOR_LIME100: Vec4 = vec4(0.941_176_5, 0.956_862_75, 0.764_705_9, 1.0);
pub const COLOR_LIME200: Vec4 = vec4(0.901_960_8, 0.933_333_34, 0.611_764_7, 1.0);
pub const COLOR_LIME300: Vec4 = vec4(0.862_745_1, 0.905_882_36, 0.458_823_53, 1.0);
pub const COLOR_LIME400: Vec4 = vec4(0.831_372_56, 0.882_352_95, 0.341_176_48, 1.0);
pub const COLOR_LIME600: Vec4 = vec4(0.752_941_2, 0.792_156_9, 0.2, 1.0);
pub const COLOR_LIME700: Vec4 = vec4(0.686_274_5, 0.705_882_4, 0.168_627_46, 1.0);
pub const COLOR_LIME800: Vec4 = vec4(0.619_607_87, 0.615_686_3, 0.141_176_48, 1.0);
pub const COLOR_LIME900: Vec4 = vec4(0.509_803_95, 0.466_666_67, 0.090_196_08, 1.0);
pub const COLOR_LIMEA100: Vec4 = vec4(0.956_862_75, 1.0, 0.505_882_4, 1.0);
pub const COLOR_LIMEA200: Vec4 = vec4(0.933_333_34, 1.0, 0.254_901_98, 1.0);
pub const COLOR_LIMEA400: Vec4 = vec4(0.776_470_6, 1.0, 0.0, 1.0);
pub const COLOR_LIMEA700: Vec4 = vec4(0.682_352_96, 0.917_647_06, 0.0, 1.0);
pub const COLOR_YELLOW500: Vec4 = vec4(1.0, 0.921_568_63, 0.231_372_55, 1.0);
pub const COLOR_YELLOW50: Vec4 = vec4(1.0, 0.992_156_86, 0.905_882_36, 1.0);
pub const COLOR_YELLOW100: Vec4 = vec4(1.0, 0.976_470_6, 0.768_627_46, 1.0);
pub const COLOR_YELLOW200: Vec4 = vec4(1.0, 0.960_784_3, 0.615_686_3, 1.0);
pub const COLOR_YELLOW300: Vec4 = vec4(1.0, 0.945_098_04, 0.462_745_1, 1.0);
pub const COLOR_YELLOW400: Vec4 = vec4(1.0, 0.933_333_34, 0.345_098_05, 1.0);
pub const COLOR_YELLOW600: Vec4 = vec4(0.992_156_86, 0.847_058_83, 0.207_843_14, 1.0);
pub const COLOR_YELLOW700: Vec4 = vec4(0.984_313_7, 0.752_941_2, 0.176_470_6, 1.0);
pub const COLOR_YELLOW800: Vec4 = vec4(0.976_470_6, 0.658_823_55, 0.145_098_05, 1.0);
pub const COLOR_YELLOW900: Vec4 = vec4(0.960_784_3, 0.498_039_22, 0.090_196_08, 1.0);
pub const COLOR_YELLOWA100: Vec4 = vec4(1.0, 1.0, 0.552_941_2, 1.0);
pub const COLOR_YELLOWA200: Vec4 = vec4(1.0, 1.0, 0.0, 1.0);
pub const COLOR_YELLOWA400: Vec4 = vec4(1.0, 0.917_647_06, 0.0, 1.0);
pub const COLOR_YELLOWA700: Vec4 = vec4(1.0, 0.839_215_7, 0.0, 1.0);
pub const COLOR_AMBER500: Vec4 = vec4(1.0, 0.756_862_76, 0.027_450_98, 1.0);
pub const COLOR_AMBER50: Vec4 = vec4(1.0, 0.972_549, 0.882_352_95, 1.0);
pub const COLOR_AMBER100: Vec4 = vec4(1.0, 0.925_490_2, 0.701_960_8, 1.0);
pub const COLOR_AMBER200: Vec4 = vec4(1.0, 0.878_431_4, 0.509_803_95, 1.0);
pub const COLOR_AMBER300: Vec4 = vec4(1.0, 0.835_294_1, 0.309_803_93, 1.0);
pub const COLOR_AMBER400: Vec4 = vec4(1.0, 0.792_156_9, 0.156_862_75, 1.0);
pub const COLOR_AMBER600: Vec4 = vec4(1.0, 0.701_960_8, 0.0, 1.0);
pub const COLOR_AMBER700: Vec4 = vec4(1.0, 0.627_451, 0.0, 1.0);
pub const COLOR_AMBER800: Vec4 = vec4(1.0, 0.560_784_34, 0.0, 1.0);
pub const COLOR_AMBER900: Vec4 = vec4(1.0, 0.435_294_12, 0.0, 1.0);
pub const COLOR_AMBERA100: Vec4 = vec4(1.0, 0.898_039_2, 0.498_039_22, 1.0);
pub const COLOR_AMBERA200: Vec4 = vec4(1.0, 0.843_137_26, 0.250_980_4, 1.0);
pub const COLOR_AMBERA400: Vec4 = vec4(1.0, 0.768_627_46, 0.0, 1.0);
pub const COLOR_AMBERA700: Vec4 = vec4(1.0, 0.670_588_25, 0.0, 1.0);
pub const COLOR_ORANGE500: Vec4 = vec4(1.0, 0.596_078_46, 0.0, 1.0);
pub const COLOR_ORANGE50: Vec4 = vec4(1.0, 0.952_941_2, 0.878_431_4, 1.0);
pub const COLOR_ORANGE100: Vec4 = vec4(1.0, 0.878_431_4, 0.698_039_23, 1.0);
pub const COLOR_ORANGE200: Vec4 = vec4(1.0, 0.8, 0.501_960_8, 1.0);
pub const COLOR_ORANGE300: Vec4 = vec4(1.0, 0.717_647_1, 0.301_960_8, 1.0);
pub const COLOR_ORANGE400: Vec4 = vec4(1.0, 0.654_902, 0.149_019_61, 1.0);
pub const COLOR_ORANGE600: Vec4 = vec4(0.984_313_7, 0.549_019_63, 0.0, 1.0);
pub const COLOR_ORANGE700: Vec4 = vec4(0.960_784_3, 0.486_274_5, 0.0, 1.0);
pub const COLOR_ORANGE800: Vec4 = vec4(0.937_254_9, 0.423_529_42, 0.0, 1.0);
pub const COLOR_ORANGE900: Vec4 = vec4(0.901_960_8, 0.317_647_07, 0.0, 1.0);
pub const COLOR_ORANGEA100: Vec4 = vec4(1.0, 0.819_607_85, 0.501_960_8, 1.0);
pub const COLOR_ORANGEA200: Vec4 = vec4(1.0, 0.670_588_25, 0.250_980_4, 1.0);
pub const COLOR_ORANGEA400: Vec4 = vec4(1.0, 0.568_627_5, 0.0, 1.0);
pub const COLOR_ORANGEA700: Vec4 = vec4(1.0, 0.427_450_98, 0.0, 1.0);
pub const COLOR_DEEPORANGE500: Vec4 = vec4(1.0, 0.341_176_48, 0.133_333_34, 1.0);
pub const COLOR_DEEPORANGE50: Vec4 = vec4(0.984_313_7, 0.913_725_5, 0.905_882_36, 1.0);
pub const COLOR_DEEPORANGE100: Vec4 = vec4(1.0, 0.8, 0.737_254_9, 1.0);
pub const COLOR_DEEPORANGE200: Vec4 = vec4(1.0, 0.670_588_25, 0.568_627_5, 1.0);
pub const COLOR_DEEPORANGE300: Vec4 = vec4(1.0, 0.541_176_5, 0.396_078_44, 1.0);
pub const COLOR_DEEPORANGE400: Vec4 = vec4(1.0, 0.439_215_7, 0.262_745_1, 1.0);
pub const COLOR_DEEPORANGE600: Vec4 = vec4(0.956_862_75, 0.317_647_07, 0.117_647_06, 1.0);
pub const COLOR_DEEPORANGE700: Vec4 = vec4(0.901_960_8, 0.290_196_1, 0.098_039_22, 1.0);
pub const COLOR_DEEPORANGE800: Vec4 = vec4(0.847_058_83, 0.262_745_1, 0.082_352_94, 1.0);
pub const COLOR_DEEPORANGE900: Vec4 = vec4(0.749_019_6, 0.211_764_71, 0.047_058_824, 1.0);
pub const COLOR_DEEPORANGEA100: Vec4 = vec4(1.0, 0.619_607_87, 0.501_960_8, 1.0);
pub const COLOR_DEEPORANGEA200: Vec4 = vec4(1.0, 0.431_372_55, 0.250_980_4, 1.0);
pub const COLOR_DEEPORANGEA400: Vec4 = vec4(1.0, 0.239_215_69, 0.0, 1.0);
pub const COLOR_DEEPORANGEA700: Vec4 = vec4(0.866_666_7, 0.172_549_02, 0.0, 1.0);
pub const COLOR_BROWN500: Vec4 = vec4(0.474_509_8, 0.333_333_34, 0.282_352_95, 1.0);
pub const COLOR_BROWN50: Vec4 = vec4(0.937_254_9, 0.921_568_63, 0.913_725_5, 1.0);
pub const COLOR_BROWN100: Vec4 = vec4(0.843_137_26, 0.8, 0.784_313_74, 1.0);
pub const COLOR_BROWN200: Vec4 = vec4(0.737_254_9, 0.666_666_7, 0.643_137_3, 1.0);
pub const COLOR_BROWN300: Vec4 = vec4(0.631_372_6, 0.533_333_36, 0.498_039_22, 1.0);
pub const COLOR_BROWN400: Vec4 = vec4(0.552_941_2, 0.431_372_55, 0.388_235_3, 1.0);
pub const COLOR_BROWN600: Vec4 = vec4(0.427_450_98, 0.298_039_23, 0.254_901_98, 1.0);
pub const COLOR_BROWN700: Vec4 = vec4(0.364_705_9, 0.250_980_4, 0.215_686_28, 1.0);
pub const COLOR_BROWN800: Vec4 = vec4(0.305_882_36, 0.203_921_57, 0.180_392_16, 1.0);
pub const COLOR_BROWN900: Vec4 = vec4(0.243_137_26, 0.152_941_18, 0.137_254_91, 1.0);
pub const COLOR_GREY500: Vec4 = vec4(0.619_607_87, 0.619_607_87, 0.619_607_87, 1.0);
pub const COLOR_GREY50: Vec4 = vec4(0.980_392_16, 0.980_392_16, 0.980_392_16, 1.0);
pub const COLOR_GREY100: Vec4 = vec4(0.960_784_3, 0.960_784_3, 0.960_784_3, 1.0);
pub const COLOR_GREY200: Vec4 = vec4(0.933_333_34, 0.933_333_34, 0.933_333_34, 1.0);
pub const COLOR_GREY300: Vec4 = vec4(0.878_431_4, 0.878_431_4, 0.878_431_4, 1.0);
pub const COLOR_GREY400: Vec4 = vec4(0.741_176_5, 0.741_176_5, 0.741_176_5, 1.0);
pub const COLOR_GREY600: Vec4 = vec4(0.458_823_53, 0.458_823_53, 0.458_823_53, 1.0);
pub const COLOR_GREY700: Vec4 = vec4(0.380_392_16, 0.380_392_16, 0.380_392_16, 1.0);
pub const COLOR_GREY800: Vec4 = vec4(0.258_823_54, 0.258_823_54, 0.258_823_54, 1.0);
pub const COLOR_GREY850: Vec4 = vec4(0.192_156_87, 0.192_156_87, 0.192_156_87, 1.0);
pub const COLOR_GREY900: Vec4 = vec4(0.129_411_77, 0.129_411_77, 0.129_411_77, 1.0);
pub const COLOR_BLUEGREY500: Vec4 = vec4(0.376_470_6, 0.490_196_08, 0.545_098_07, 1.0);
pub const COLOR_BLUEGREY50: Vec4 = vec4(0.925_490_2, 0.937_254_9, 0.945_098_04, 1.0);
pub const COLOR_BLUEGREY100: Vec4 = vec4(0.811_764_7, 0.847_058_83, 0.862_745_1, 1.0);
pub const COLOR_BLUEGREY200: Vec4 = vec4(0.690_196_1, 0.745_098_05, 0.772_549_03, 1.0);
pub const COLOR_BLUEGREY300: Vec4 = vec4(0.564_705_9, 0.643_137_3, 0.682_352_96, 1.0);
pub const COLOR_BLUEGREY400: Vec4 = vec4(0.470_588_24, 0.564_705_9, 0.611_764_7, 1.0);
pub const COLOR_BLUEGREY600: Vec4 = vec4(0.329_411_77, 0.431_372_55, 0.478_431_37, 1.0);
pub const COLOR_BLUEGREY700: Vec4 = vec4(0.270_588_25, 0.352_941_2, 0.392_156_87, 1.0);
pub const COLOR_BLUEGREY800: Vec4 = vec4(0.215_686_28, 0.278_431_4, 0.309_803_93, 1.0);
pub const COLOR_BLUEGREY900: Vec4 = vec4(0.149_019_61, 0.196_078_43, 0.219_607_84, 1.0);