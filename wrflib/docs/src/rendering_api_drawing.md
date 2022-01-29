# Drawing

In a `draw()` function, [`Cx`](/target/doc/wrflib/struct.Cx.html) provides a few different functions to actually render to the screen.
 * Use [`add_instances`](/target/doc/wrflib/struct.Cx.html#method.add_instances) to render with a [`Shader`](/target/doc/wrflib/struct.Shader.html) and instance data. This will use the shader's `build_geom` as the rendered geometry.
 * Use [`add_mesh_instances`](/target/doc/wrflib/struct.Cx.html#method.add_mesh_instances) to render with a custom geometry, passing in a [`GpuGeometry`](/target/doc/wrflib/struct.GpuGeometry.html).
 * Use [`add_instances_with_scroll_sticky`](/target/doc/wrflib/struct.Cx.html#method.add_instances_with_scroll_sticky) to disable default scrolling behavior and keep items sticky on the screen. This is only relevant for 2D rendering that respects scrolling, such as UI components.
