use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Key, Transform, logger},
    game::{AppError, Application, GameContext, GameOptions, Run},
    render::{
        MaterialInstanceProperties, MaterialsConfig, Model, RenderError, TextureProperties,
        TextureType,
    },
    resource::{
        entity::{camera::Camera, light::Light},
        geometry::Shapes,
        resource::ResourceLifetime,
    },
    scene::{
        components::{NodeId, NodeValue},
        graph::scenegraph::SceneGraph,
        scene::Scene,
    },
    ui::UIRenderer,
};

use examples::{CameraController, SampleApp};

struct App {
    common: SampleApp,
    camera_controller: CameraController,
    ui: UIRenderer,
    scene: Scene,
    nodes: Vec<NodeId>,
}

impl Run for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let extent = ctx.renderer.extent();

        let camera = Camera::perspective(
            extent.width as f32 / extent.height as f32,
            60_f32.to_radians(),
            0.1,
            100.,
            0.,
            0.,
        );
        let camera_position = Vec3::new(0., 0., 2.5);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 0., 2.);

        let common = SampleApp::new();

        let camera_controller = SampleApp::controller();

        let ui = UIRenderer::new(&ctx.renderer.gfx, &mut ctx.resource_manager, true)?;
        let scene = Scene::new(
            &ctx.renderer.gfx,
            camera,
            camera_position,
            light,
            light_position,
        );

        Ok(App {
            common,
            camera_controller,
            ui,
            scene,
            nodes: vec![],
        })
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        if self.common.process_updates {
            let angular_speed = 10.;
            self.scene.graph.update(self.nodes[2], |node| {
                node.update_transform(|transform| {
                    transform.rotate(Quat::from_axis_angle(
                        Vec3::Y,
                        (angular_speed * delta).to_radians(),
                    ));

                    true
                });

                false
            });
        }

        self.scene.update_camera(|transform, camera| {
            self.camera_controller
                .update_camera(camera, transform, delta)
        });

        self.scene.update(&ctx.renderer.gfx, delta);

        self.common
            .update_ui(ctx, &mut self.scene, &mut self.ui, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        self.common
            .render(ctx, Some(&mut self.scene), Some(&mut self.ui))
    }

    fn input(&mut self, ctx: &mut GameContext, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.scene,
            &mut self.ui,
            Some(&mut self.camera_controller),
        );

        if let Input::KeyPressed(key) = input {
            match key {
                Key::N0 => {
                    self.scene.graph.toggle(self.nodes[0]);
                }
                Key::N1 => {
                    self.scene.graph.toggle(self.nodes[1]);
                }
                Key::N2 => {
                    self.scene.graph.toggle(self.nodes[2]);
                }
                _ => (),
            }
        }
    }

    fn resize(&mut self, _ctx: &mut GameContext, width: u32, height: u32) {
        self.scene.resize(width, height);
        self.ui.resize(width, height);
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &mut GameContext) {
        MaterialsConfig::load_resources(
            &ctx.renderer.gfx,
            "materials.ron",
            &mut ctx.resource_manager,
        )
        .await;

        let properties = TextureProperties::with_file("Wall Diffuse", examples::WALL_TEXTURE);
        let diffuse_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static);

        let mut properties = TextureProperties::with_file("Wall Normal", examples::WALL_TEXTURE_N);
        properties.format.ty = TextureType::Normal;
        let normal_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static);

        let material = ctx.resource_manager.get_by_name("normal").unwrap();
        let material_instance_properties = MaterialInstanceProperties::new("normal", material)
            .textures(&[diffuse_texture, normal_texture]);
        let material_instance = ctx
            .resource_manager
            .add(material_instance_properties, ResourceLifetime::Static);

        let cube = Model::builder("cube")
            .mesh(
                Shapes::cubemap(1, 1, &[1], 1., ctx.renderer.gfx.vertex_padding),
                Some(material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .build();

        let graph = &mut self.scene.graph;

        /*
                                    0
                            /               \
                        1                       2
                    /       \               /       \
                3               4       5               6
            /   |   \                                   |
            7   8   9                                   10
            |
            11
        */

        let node_value = NodeValue::Model(cube);

        let dx = 1.4;
        let dy = 1.4;

        let mut root_transform = Transform::from_translation([0., 0.6 * dy, 0.].into());
        root_transform.scale(Vec3::splat(0.3));

        let node0 = graph
            .insert(graph.root, node_value.clone(), root_transform)
            .unwrap();
        self.nodes.push(node0);

        let mut subgraph1 = SceneGraph::new();
        let node1 = subgraph1.set_root(
            node_value.clone(),
            Transform::from_translation([-2. * dx, -dy, 0.].into()),
        );

        let mut subgraph2 = SceneGraph::new();
        let node2 = subgraph2.set_root(
            node_value.clone(),
            Transform::from_translation([2. * dx, -dy, 0.].into()),
        );

        let node3 = subgraph1
            .insert(
                node1,
                node_value.clone(),
                Transform::from_translation([-dx, -dy, 0.].into()),
            )
            .unwrap();

        let _node4 = subgraph1
            .insert(
                node1,
                node_value.clone(),
                Transform::from_translation([dx, -dy, 0.].into()),
            )
            .unwrap();

        let _node5 = subgraph2
            .insert(
                node2,
                node_value.clone(),
                Transform::from_translation([-dx, -dy, 0.].into()),
            )
            .unwrap();

        let node6 = subgraph2
            .insert(
                node2,
                node_value.clone(),
                Transform::from_translation([dx, -dy, 0.].into()),
            )
            .unwrap();

        let node7 = subgraph1
            .insert(
                node3,
                node_value.clone(),
                Transform::from_translation([-dx, -dy, 0.].into()),
            )
            .unwrap();

        let _node8 = subgraph1
            .insert(
                node3,
                node_value.clone(),
                Transform::from_translation([0., -dy, 0.].into()),
            )
            .unwrap();

        let _node9 = subgraph1
            .insert(
                node3,
                node_value.clone(),
                Transform::from_translation([dx, -dy, 0.].into()),
            )
            .unwrap();

        let _node10 = subgraph2
            .insert(
                node6,
                node_value.clone(),
                Transform::from_translation([0., -dy, 0.].into()),
            )
            .unwrap();

        let _node11 = subgraph1
            .insert(
                node7,
                node_value.clone(),
                Transform::from_translation([0., -dy, 0.].into()),
            )
            .unwrap();

        self.nodes.push(
            graph
                .insert_subgraph(node0, subgraph1.root, &subgraph1)
                .unwrap(),
        );
        self.nodes.push(
            graph
                .insert_subgraph(node0, subgraph2.root, &subgraph2)
                .unwrap(),
        );
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App>::new(
        "Scenegraph",
        GameOptions::default(),
        examples::WIDTH,
        examples::HEIGHT,
    )
    .run();
}
