use futures::try_join;
use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Transform},
    game::{
        app::{Application, Run},
        input::{Input, Key},
    },
    gfx::{Device, SamplerFilter},
    render::{
        context::Context,
        graph::{FrameGraph, RenderError},
        material::{Texture, TextureType},
        pass::PassType,
        renderable::Renderable,
        Model,
    },
    resource::entity::{camera::Camera, light::Light},
    scene::{
        components::{NodeId, NodeValue},
        graph::scenegraph::SceneGraph,
        scene::Scene,
        shape::Shapes,
    },
    ui::UIRenderer,
};

use examples::{CameraController, SampleApp};

struct App {
    common: SampleApp,
    camera_controller: CameraController,
    graph: FrameGraph,
    ui: UIRenderer,
    scene: Scene,
    nodes: Vec<NodeId>,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let extent = ctx.extent();

        let camera = Camera::perspective(
            extent.width as f32 / extent.height as f32,
            (60. as f32).to_radians(),
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

        let graph = FrameGraph::default(ctx);
        let ui = UIRenderer::new(ctx, graph.pass_by_type(PassType::Ui).unwrap());
        let scene = Scene::new(camera, camera_position, light, light_position);

        App {
            common,
            camera_controller,
            graph,
            ui,
            scene,
            nodes: vec![],
        }
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        if self.common.process_updates {
            let angular_speed = 10.;
            self.scene.graph.update(self.nodes[2], |node| {
                node.update_transform(|transform| {
                    transform.rotate(Quat::from_axis_angle(
                        Vec3::Y,
                        (angular_speed * delta).to_radians(),
                    ))
                });
            });
        }

        self.scene.update_camera(|transform, camera| {
            self.camera_controller
                .update_camera(camera, transform, delta);
        });

        self.graph.update(ctx, delta);
        self.scene.update(ctx, delta);

        self.common
            .update_ui(ctx, &self.graph, &self.scene, &mut self.ui, delta);
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        self.common
            .render(ctx, &mut self.graph, &mut self.scene, &mut self.ui)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.graph,
            &mut self.scene,
            &mut self.ui,
            &mut self.camera_controller,
        );

        match input {
            Input::KeyPressed(key) => match key {
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
            },
            _ => (),
        }
    }

    fn resize(&mut self, ctx: &mut Context, width: u32, height: u32) {
        self.graph.resize(ctx);
        self.scene.resize(width, height);
        self.ui.resize(width, height);
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx).await;
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &Context) {
        let material = self.common.normal_mapping_material(ctx, &self.graph);

        let diffuse_texture = Texture::with_file(
            ctx,
            examples::WALL_TEXTURE,
            TextureType::Diffuse,
            SamplerFilter::FilterLinear,
            SamplerFilter::FilterLinear,
        );

        let normal_texture = Texture::with_file(
            ctx,
            examples::WALL_TEXTURE_N,
            TextureType::Normal,
            SamplerFilter::FilterLinear,
            SamplerFilter::FilterLinear,
        );

        let (diffuse_texture, normal_texture) = try_join!(diffuse_texture, normal_texture).unwrap();

        let material_instance = material.instantiate(vec![diffuse_texture, normal_texture]);

        let cube = Model::builder("cube")
            .mesh(Shapes::cubemap(1, 1, &[1], 1.), Some(material_instance))
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

    log::info!("Engine start");

    Application::<App>::new("Scenegraph", examples::WIDTH, examples::HEIGHT).run();
}
