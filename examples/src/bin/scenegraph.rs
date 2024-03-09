use gobs::{
    core::{entity::light::Light, Color, Transform},
    game::{
        app::{Application, Run},
        input::{Input, Key},
    },
    render::{
        context::Context,
        geometry::Model,
        graph::{FrameGraph, RenderError},
        pass::PassType,
        renderable::Renderable,
    },
    scene::{
        graph::scenegraph::{NodeId, NodeValue, SceneGraph},
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
        let light = Light::new((0., 0., 10.), Color::WHITE);
        let camera = SampleApp::ortho_camera(ctx);

        let common = SampleApp::new();

        let camera_controller = SampleApp::controller();

        let graph = FrameGraph::default(ctx);
        let ui = UIRenderer::new(ctx, graph.pass_by_type(PassType::Ui).unwrap());
        let scene = Scene::new(camera, light);

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
        self.camera_controller
            .update_camera(&mut self.scene.camera_mut(), delta);

        self.graph.update(ctx, delta);
        self.scene.update(ctx, delta);

        self.common
            .update_ui(ctx, &self.graph, &self.scene, &mut self.ui, |_| {});
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        self.common
            .render(ctx, &mut self.graph, &mut self.scene, &mut self.ui)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.graph,
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

    fn resize(&mut self, ctx: &Context, width: u32, height: u32) {
        self.graph.resize(ctx);
        self.scene.resize(width, height);
        self.ui.resize(width, height);
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx);
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    fn init(&mut self, ctx: &Context) {
        let material = self.common.color_material(ctx, &self.graph);
        let material_instance = material.instantiate(vec![]);

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(Color::RED, Color::GREEN, Color::BLUE, 1.),
                material_instance,
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

        let node_value = NodeValue::Model(triangle);

        let extent = ctx.surface.get_extent(ctx.device.clone());
        let dx = extent.width as f32 / 12.;
        let dy = extent.height as f32 / 6.;

        let mut root_transform = Transform::translation([0., 2. * dy, 0.].into());
        root_transform.scale([100., 100., 1.].into());

        let node0 = graph
            .insert(graph.root, node_value.clone(), root_transform)
            .unwrap();
        self.nodes.push(node0);

        let mut subgraph1 = SceneGraph::new();
        let node1 = subgraph1.set_root(
            node_value.clone(),
            Transform::translation([-2. * dx, -dy, 0.].into()),
        );

        let mut subgraph2 = SceneGraph::new();
        let node2 = subgraph2.set_root(
            node_value.clone(),
            Transform::translation([2. * dx, -dy, 0.].into()),
        );

        let node3 = subgraph1
            .insert(
                node1,
                node_value.clone(),
                Transform::translation([-dx, -dy, 0.].into()),
            )
            .unwrap();

        let _node4 = subgraph1
            .insert(
                node1,
                node_value.clone(),
                Transform::translation([dx, -dy, 0.].into()),
            )
            .unwrap();

        let _node5 = subgraph2
            .insert(
                node2,
                node_value.clone(),
                Transform::translation([-dx, -dy, 0.].into()),
            )
            .unwrap();

        let node6 = subgraph2
            .insert(
                node2,
                node_value.clone(),
                Transform::translation([dx, -dy, 0.].into()),
            )
            .unwrap();

        let node7 = subgraph1
            .insert(
                node3,
                node_value.clone(),
                Transform::translation([-dx, -dy, 0.].into()),
            )
            .unwrap();

        let _node8 = subgraph1
            .insert(
                node3,
                node_value.clone(),
                Transform::translation([0., -dy, 0.].into()),
            )
            .unwrap();

        let _node9 = subgraph1
            .insert(
                node3,
                node_value.clone(),
                Transform::translation([dx, -dy, 0.].into()),
            )
            .unwrap();

        let _node10 = subgraph2
            .insert(
                node6,
                node_value.clone(),
                Transform::translation([0., -dy, 0.].into()),
            )
            .unwrap();

        let _node11 = subgraph1
            .insert(
                node7,
                node_value.clone(),
                Transform::translation([0., -dy, 0.].into()),
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

    Application::new("Scenegraph", 1920, 1080).run::<App>();
}
