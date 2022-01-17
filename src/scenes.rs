use anyhow::Result;
use glam::{IVec2, Mat4, Vec3};

use crab_tv::{
    look_at_transform, viewport_transform, Canvas, Model, ModelShading, BLUE, CYAN, GREEN, RED,
    WHITE,
};
use strum::IntoEnumIterator;

use crate::shaders::PhongShadowInput;

#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumIter,
    PartialEq,
    Eq,
    strum::Display,
)]
#[strum(serialize_all = "title_case")]
pub enum RenderScene {
    FivePixels,
    Lines,
    ModelWireframe,
    TriangleLineSweepVerbose,
    TriangleLineSweepCompact,
    TriangleBarycentric,
    ModelColoredTriangles,
    ModelFlatShaded,
    DepthBuffer,
    ModelDepthTested,
    ModelTextured,
    ModelPerspective,
    ModelGouraud,
    MovableCamera,
    ReimplementAsShader,
    GouraudIntensitiesBucketed,
    NormalGlobalAsDiffuse,
    NormalShader,
    NormalTangentAsDiffuse,
    PhongShader,
    ShadowBuffer,
    Shadowed,
    ScreenSpaceAmbientOcclusionCalculated,
    ScreenSpaceAmbientOcclusion,
}

impl Default for RenderScene {
    fn default() -> Self {
        RenderScene::iter().last().unwrap()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_scene(
    image: &mut Canvas,
    scene: &RenderScene,
    model: &Model,
    light_dir: Vec3,
    camera_distance: f32,
    camera_look_from: Vec3,
    camera_look_at: Vec3,
    camera_up: Vec3,
    ambient_occlusion_passes: usize,
    ambient_occlusion_strength: f32,
) -> Result<()> {
    println!("Rendering scene: {}", scene);

    let viewport = viewport_transform(
        image.width() as f32 / 8.0,
        image.height() as f32 / 8.0,
        image.width() as f32 * 3.0 / 4.0,
        image.height() as f32 * 3.0 / 4.0,
    );

    // projection matrix applies perspective correction
    let projection_transform = Mat4::from_cols(
        [1.0, 0.0, 0.0, 0.0].into(),
        [0.0, 1.0, 0.0, 0.0].into(),
        [0.0, 0.0, 1.0, -1.0 / camera_distance].into(),
        [0.0, 0.0, 0.0, 1.0].into(),
    );

    let model_view_transform = look_at_transform(camera_look_from, camera_look_at, camera_up);

    let uniform_m = projection_transform * model_view_transform;

    match scene {
        RenderScene::FivePixels => {
            // pixel in the middle
            *image.pixel_mut(image.width() as i32 / 2, image.height() as i32 / 2) = WHITE;
            // then each of the 4 corners
            *image.pixel_mut(0, image.height() as i32 - 1) = RED; // top left
            *image.pixel_mut(image.width() as i32 - 1, image.height() as i32 - 1) = GREEN; // top right
            *image.pixel_mut(0, 0) = BLUE; // bottom left
            *image.pixel_mut(image.width() as i32 - 1, 0) = CYAN; // bottom right
        }
        RenderScene::Lines => {
            image.line(IVec2::new(13, 20), IVec2::new(80, 40), WHITE);
            image.line(IVec2::new(20, 13), IVec2::new(40, 80), RED);
            image.line(IVec2::new(80, 40), IVec2::new(13, 20), BLUE);
            image.line(IVec2::new(0, 0), IVec2::new(50, 50), GREEN);
        }
        RenderScene::ModelWireframe => {
            image.model_wireframe(model, WHITE);
        }
        RenderScene::TriangleLineSweepVerbose => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle_linesweep_verbose(&t0, RED);
            image.triangle_linesweep_verbose(&t1, WHITE);
            image.triangle_linesweep_verbose(&t2, GREEN);
        }
        RenderScene::TriangleLineSweepCompact => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle_linesweep_compact(&t0, RED);
            image.triangle_linesweep_compact(&t1, WHITE);
            image.triangle_linesweep_compact(&t2, GREEN);
        }
        RenderScene::TriangleBarycentric => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle_barycentric(&t0, RED);
            image.triangle_barycentric(&t1, WHITE);
            image.triangle_barycentric(&t2, GREEN);
        }
        RenderScene::ModelColoredTriangles => {
            image.model_colored_triangles(model);
        }
        RenderScene::ModelFlatShaded => {
            image.model_fixed_function(model, light_dir, ModelShading::FlatOnly, None);
        }
        RenderScene::DepthBuffer => {
            image.model_fixed_function(model, light_dir, ModelShading::DepthTested, None);
            image.replace_with_z_buffer();
        }
        RenderScene::ModelDepthTested => {
            image.model_fixed_function(model, light_dir, ModelShading::DepthTested, None);
        }
        RenderScene::ModelTextured => {
            image.model_fixed_function(model, light_dir, ModelShading::Textured, None)
        }
        RenderScene::ModelPerspective => image.model_fixed_function(
            model,
            light_dir,
            ModelShading::Textured,
            Some(projection_transform),
        ),
        RenderScene::ModelGouraud => image.model_fixed_function(
            model,
            light_dir,
            ModelShading::Gouraud,
            Some(projection_transform),
        ),
        RenderScene::MovableCamera => image.model_fixed_function(
            model,
            light_dir,
            ModelShading::Gouraud,
            Some(projection_transform * model_view_transform),
        ),
        RenderScene::ReimplementAsShader => {
            let shader = crate::shaders::GouraudShader::new(
                viewport,
                uniform_m,
                light_dir,
                Some(&model.diffuse_texture),
                false,
            );

            image.model_shader(model, &shader);
        }
        RenderScene::GouraudIntensitiesBucketed => {
            let shader = crate::shaders::GouraudShader::new(
                viewport,
                uniform_m,
                light_dir,
                Some(&model.diffuse_texture),
                true,
            );

            image.model_shader(model, &shader);
        }
        RenderScene::NormalGlobalAsDiffuse => {
            let shader =
                crate::shaders::UnlitShader::new(viewport, uniform_m, &model.normal_texture_global);

            image.model_shader(model, &shader);
        }
        RenderScene::NormalShader => {
            let shader = crate::shaders::NormalShader::new(
                viewport,
                uniform_m,
                light_dir,
                &model.diffuse_texture,
                &model.normal_texture_global,
            );

            image.model_shader(model, &shader);
        }
        RenderScene::NormalTangentAsDiffuse => {
            let shader = crate::shaders::UnlitShader::new(
                viewport,
                uniform_m,
                &model.normal_texture_darboux,
            );

            image.model_shader(model, &shader);
        }
        RenderScene::PhongShader => {
            let shader = crate::shaders::PhongShader::new(
                viewport,
                uniform_m,
                light_dir,
                &model.diffuse_texture,
                &model.normal_texture_global,
                &model.specular_texture,
                None,
            );

            image.model_shader(model, &shader);
        }
        RenderScene::ShadowBuffer => {
            let shader = crate::shaders::DepthShader::new(
                viewport,
                // NB: looking from the light position so that framebuffer is filled with shadow buffer
                look_at_transform(light_dir, camera_look_at, camera_up),
            );

            image.model_shader(model, &shader);
        }
        RenderScene::Shadowed => {
            let mut shadow_buffer = image.clone();

            let shadow_modelview_transform =
                look_at_transform(light_dir, camera_look_at, camera_up);
            let shadow_projection = Mat4::IDENTITY;
            let shadow_buffer_shader = crate::shaders::DepthShader::new(
                viewport,
                shadow_projection * shadow_modelview_transform,
            );
            shadow_buffer.model_shader(model, &shadow_buffer_shader);
            let shadow_m = viewport * shadow_projection * shadow_modelview_transform;

            let shader = crate::shaders::PhongShader::new(
                viewport,
                uniform_m,
                light_dir,
                &model.diffuse_texture,
                &model.normal_texture_global,
                &model.specular_texture,
                Some(PhongShadowInput::new(
                    shadow_m * (viewport * uniform_m).inverse(),
                    shadow_buffer,
                )),
            );

            image.model_shader(model, &shader);
        }
        RenderScene::ScreenSpaceAmbientOcclusionCalculated => {
            let z_depth_shader = crate::shaders::PureColorShader::new(viewport, uniform_m);
            image.model_shader(model, &z_depth_shader);

            image.apply_ambient_occlusion(10.0, ambient_occlusion_passes)
        }
        RenderScene::ScreenSpaceAmbientOcclusion => {
            let mut shadow_buffer = image.clone();

            let shadow_modelview_transform =
                look_at_transform(light_dir, camera_look_at, camera_up);
            let shadow_projection = Mat4::IDENTITY;
            let shadow_buffer_shader = crate::shaders::DepthShader::new(
                viewport,
                shadow_projection * shadow_modelview_transform,
            );
            shadow_buffer.model_shader(model, &shadow_buffer_shader);
            let shadow_m = viewport * shadow_projection * shadow_modelview_transform;

            let shader = crate::shaders::PhongShader::new(
                viewport,
                uniform_m,
                light_dir,
                &model.diffuse_texture,
                &model.normal_texture_global,
                &model.specular_texture,
                Some(PhongShadowInput::new(
                    shadow_m * (viewport * uniform_m).inverse(),
                    shadow_buffer,
                )),
            );

            image.model_shader(model, &shader);
            image.apply_ambient_occlusion(ambient_occlusion_strength, ambient_occlusion_passes)
        }
    }

    image.flip_y();

    Ok(())
}
#[cfg(test)]
mod tests {
    use std::path::Path;

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn every_scene_should_render_without_errors() -> Result<()> {
        for scene in RenderScene::iter() {
            let mut image = Canvas::new(200, 200);
            println!("Rendering scene: {:?}", scene);
            render_scene(
                &mut image,
                &scene,
                &Model::load_obj_file(&Model::validate(
                    Path::new("assets/african_head.obj").as_ref(),
                )?)
                .expect("model load should succeed"),
                Vec3::new(0.0, 0.0, -1.0),
                3.0,
                Vec3::new(0.0, 0.0, 3.0),
                Vec3::ZERO,
                Vec3::new(0.0, 1.0, 0.0),
                5,
                2.0,
            )?;
        }
        Ok(())
    }
}
