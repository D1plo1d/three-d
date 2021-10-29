use crate::core::*;
use crate::renderer::*;

///
/// A triangle mesh which can be rendered with a [ForwardMaterial] or [DeferredMaterial].
///
#[derive(Clone)]
pub struct Model<M: ForwardMaterial> {
    context: Context,
    mesh: Mesh,
    #[deprecated = "set in render states on material instead"]
    pub cull: Cull,
    aabb: AxisAlignedBoundingBox,
    aabb_local: AxisAlignedBoundingBox,
    transformation: Mat4,
    normal_transformation: Mat4,
    /// The material applied to the model
    pub material: M,
}

#[allow(deprecated)]
impl<M: ForwardMaterial> Model<M> {
    pub fn new(context: &Context, cpu_mesh: &CPUMesh, material: M) -> ThreeDResult<Self> {
        let mesh = Mesh::new(context, cpu_mesh)?;
        let aabb = cpu_mesh.compute_aabb();
        Ok(Self {
            mesh,
            aabb,
            aabb_local: aabb.clone(),
            transformation: Mat4::identity(),
            normal_transformation: Mat4::identity(),
            context: context.clone(),
            cull: Cull::default(),
            material,
        })
    }

    pub(in crate::renderer) fn set_transformation_2d(&mut self, transformation: Mat3) {
        self.set_transformation(Mat4::new(
            transformation.x.x,
            transformation.x.y,
            0.0,
            transformation.x.z,
            transformation.y.x,
            transformation.y.y,
            0.0,
            transformation.y.z,
            0.0,
            0.0,
            1.0,
            0.0,
            transformation.z.x,
            transformation.z.y,
            0.0,
            transformation.z.z,
        ));
    }

    ///
    /// Render the mesh with a color per triangle vertex. The colors are defined when constructing the mesh and are assumed to be in gamma color space (sRGBA).
    /// Must be called in a render target render function, for example in the callback function of [Screen::write].
    /// Will render the model transparent if the colors contain alpha values below 255, you only need to render the model after all solid models.
    ///
    /// # Errors
    /// Will return an error if the mesh has no colors.
    ///
    #[deprecated = "Use 'render_forward' instead"]
    pub fn render_color(&self, camera: &Camera) -> ThreeDResult<()> {
        let mut mat = ColorMaterial::default();
        mat.opaque_render_states.cull = self.cull;
        mat.transparent_render_states.cull = self.cull;
        self.render_forward(&mat, camera, &Lights::default())
    }

    ///
    /// Render the mesh with the given color. The color is assumed to be in gamma color space (sRGBA).
    /// Must be called in a render target render function, for example in the callback function of [Screen::write].
    /// Will render the model transparent if the color contains an alpha value below 255, you only need to render the model after all solid models.
    ///
    #[deprecated = "Use 'render_forward' instead"]
    pub fn render_with_color(&self, color: Color, camera: &Camera) -> ThreeDResult<()> {
        let mut mat = ColorMaterial {
            color,
            ..Default::default()
        };
        mat.opaque_render_states.cull = self.cull;
        mat.transparent_render_states.cull = self.cull;
        self.render_forward(&mat, camera, &Lights::default())
    }

    ///
    /// Render the uv coordinates of the mesh in red (u) and green (v) for debug purposes.
    /// Must be called in a render target render function,
    /// for example in the callback function of [Screen::write].
    ///
    /// # Errors
    /// Will return an error if the mesh has no uv coordinates.
    ///
    #[deprecated = "Use 'render_forward' instead"]
    pub fn render_uvs(&self, camera: &Camera) -> ThreeDResult<()> {
        let mut mat = UVMaterial::default();
        mat.render_states.cull = self.cull;
        self.render_forward(&mat, camera, &Lights::default())
    }

    ///
    /// Render the normals of the mesh for debug purposes.
    /// Must be called in a render target render function, for example in the callback function of [Screen::write].
    ///
    /// # Errors
    /// Will return an error if the mesh has no normals.
    ///
    #[deprecated = "Use 'render_forward' instead"]
    pub fn render_normals(&self, camera: &Camera) -> ThreeDResult<()> {
        let mut mat = NormalMaterial::default();
        mat.render_states.cull = self.cull;
        self.render_forward(&mat, camera, &Lights::default())
    }

    ///
    /// Render the mesh with the given texture which is assumed to be in sRGB color space with or without an alpha channel.
    /// Must be called in a render target render function, for example in the callback function of [Screen::write].
    /// Will render the model transparent if the texture contain an alpha channel (ie. the format is [Format::RGBA]), you only need to render the model after all solid models.
    ///
    /// # Errors
    /// Will return an error if the mesh has no uv coordinates.
    ///
    #[deprecated = "Use 'render_forward' instead"]
    pub fn render_with_texture(&self, texture: &Texture2D, camera: &Camera) -> ThreeDResult<()> {
        let render_states = if texture.is_transparent() {
            RenderStates {
                cull: self.cull,
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            }
        } else {
            RenderStates {
                cull: self.cull,
                ..Default::default()
            }
        };
        let fragment_shader_source = include_str!("shaders/mesh_texture.frag");
        self.context.program(
            &Mesh::vertex_shader_source(fragment_shader_source),
            fragment_shader_source,
            |program| {
                program.use_texture("tex", texture)?;
                self.mesh.draw(
                    render_states,
                    program,
                    camera.uniform_buffer(),
                    camera.viewport(),
                    Some(self.transformation),
                    Some(self.normal_transformation),
                )
            },
        )
    }

    ///
    /// Render the depth (scaled such that a value of 1 corresponds to max_depth) into the red channel of the current color render target which for example is used for picking.
    /// Must be called in a render target render function,
    /// for example in the callback function of [Screen::write](crate::Screen::write).
    ///
    #[deprecated = "Use 'render_forward' instead"]
    pub fn render_depth_to_red(&self, camera: &Camera, max_depth: f32) -> ThreeDResult<()> {
        let mut mat = DepthMaterial {
            max_distance: Some(max_depth),
            ..Default::default()
        };
        mat.render_states.write_mask = WriteMask {
            red: true,
            ..WriteMask::DEPTH
        };
        mat.render_states.cull = self.cull;
        self.render_forward(&mat, camera, &Lights::default())
    }

    ///
    /// Render only the depth into the current depth render target which is useful for shadow maps or depth pre-pass.
    /// Must be called in a render target render function,
    /// for example in the callback function of [Screen::write](crate::Screen::write).
    ///
    #[deprecated = "Use 'render_forward' instead"]
    pub fn render_depth(&self, camera: &Camera) -> ThreeDResult<()> {
        let mut mat = DepthMaterial {
            render_states: RenderStates {
                write_mask: WriteMask::DEPTH,
                ..Default::default()
            },
            ..Default::default()
        };
        mat.render_states.cull = self.cull;
        self.render_forward(&mat, camera, &Lights::default())
    }
}

#[allow(deprecated)]
impl<M: ForwardMaterial> ShadedGeometry for Model<M> {
    fn render_with_lighting(
        &self,
        camera: &Camera,
        material: &Material,
        lighting_model: LightingModel,
        ambient_light: Option<&AmbientLight>,
        directional_lights: &[&DirectionalLight],
        spot_lights: &[&SpotLight],
        point_lights: &[&PointLight],
    ) -> ThreeDResult<()> {
        let mut mat = PhysicalMaterial::new_from_material(material)?;
        mat.opaque_render_states.cull = self.cull;
        mat.transparent_render_states.cull = self.cull;

        let mut lights: Vec<&dyn Light> = Vec::new();
        if let Some(light) = ambient_light {
            lights.push(light)
        }
        for light in directional_lights {
            lights.push(light);
        }
        for light in spot_lights {
            lights.push(light);
        }
        for light in point_lights {
            lights.push(light);
        }
        let mut fragment_shader_source =
            lights_fragment_shader_source(&mut lights.clone().into_iter(), lighting_model);
        fragment_shader_source
            .push_str(&mat.fragment_shader_source_internal(self.mesh.color_buffer.is_some()));
        self.context.program(
            &Mesh::vertex_shader_source(&fragment_shader_source),
            &fragment_shader_source,
            |program| {
                for (i, light) in lights.iter().enumerate() {
                    light.use_uniforms(program, camera, i as u32)?;
                }
                mat.use_uniforms_internal(program)?;
                self.mesh.draw(
                    mat.render_states(),
                    program,
                    camera.uniform_buffer(),
                    camera.viewport(),
                    Some(self.transformation),
                    Some(self.normal_transformation),
                )
            },
        )
    }
    fn geometry_pass(
        &self,
        camera: &Camera,
        viewport: Viewport,
        material: &Material,
    ) -> ThreeDResult<()> {
        self.render_deferred(
            &PhysicalMaterial::new_from_material(material)?,
            camera,
            viewport,
        )
    }
}

impl<M: ForwardMaterial> Geometry for Model<M> {
    fn aabb(&self) -> AxisAlignedBoundingBox {
        self.aabb
    }

    fn transformation(&self) -> Mat4 {
        self.transformation
    }
}

impl<M: ForwardMaterial> GeometryMut for Model<M> {
    fn set_transformation(&mut self, transformation: Mat4) {
        self.transformation = transformation;
        self.normal_transformation = self.transformation.invert().unwrap().transpose();
        let mut aabb = self.aabb_local.clone();
        aabb.transform(&self.transformation);
        self.aabb = aabb;
    }
}

#[allow(deprecated)]
impl<M: ForwardMaterial> Shadable for Model<M> {
    fn render_forward(
        &self,
        material: &dyn ForwardMaterial,
        camera: &Camera,
        lights: &Lights,
    ) -> ThreeDResult<()> {
        let fragment_shader_source =
            material.fragment_shader_source(self.mesh.color_buffer.is_some(), lights);
        self.context.program(
            &Mesh::vertex_shader_source(&fragment_shader_source),
            &fragment_shader_source,
            |program| {
                material.use_uniforms(program, camera, lights)?;
                self.mesh.draw(
                    material.render_states(),
                    program,
                    camera.uniform_buffer(),
                    camera.viewport(),
                    Some(self.transformation),
                    Some(self.normal_transformation),
                )
            },
        )
    }

    fn render_deferred(
        &self,
        material: &dyn DeferredMaterial,
        camera: &Camera,
        viewport: Viewport,
    ) -> ThreeDResult<()> {
        let mut render_states = material.render_states();
        render_states.cull = self.cull;
        let fragment_shader_source =
            material.fragment_shader_source_deferred(self.mesh.color_buffer.is_some());
        self.context.program(
            &Mesh::vertex_shader_source(&fragment_shader_source),
            &fragment_shader_source,
            |program| {
                material.use_uniforms(program, camera, &Lights::default())?;
                self.mesh.draw(
                    render_states,
                    program,
                    camera.uniform_buffer(),
                    viewport,
                    Some(self.transformation),
                    Some(self.normal_transformation),
                )
            },
        )
    }
}

impl<M: ForwardMaterial> Object for Model<M> {
    fn render(&self, camera: &Camera, lights: &Lights) -> ThreeDResult<()> {
        self.render_forward(&self.material, camera, lights)
    }

    fn is_transparent(&self) -> bool {
        self.material.is_transparent()
    }
}
