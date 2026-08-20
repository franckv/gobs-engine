use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Key, Transform, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    render::{RenderError, RenderType, Shapes},
    scene::{
        components::{NodeId, NodeValue},
        graph::scenegraph::SceneGraph,
        scene::Scene,
    },
};

use examples::{InputManager, Ui};

struct App<Context>
where
    Context: GobsContext,
{
    scene: Scene,
    ui: Ui<Context>,
    input: InputManager,
    nodes: Vec<NodeId>,
}

impl<Context: GobsContext> GobsGame for App<Context> {
    type Context = Context;

    async fn create(ctx: &mut Context) -> Result<Self, AppError> {
        let scene = ctx
            .new_scene()
            .with_perspective_camera(0., 0., [0., 0., 2.5])
            .with_light(Color::WHITE, [0., 0., 2.])
            .build();

        Ok(App {
            scene,
            ui: Ui::new(),
            input: InputManager::new(),
            nodes: vec![],
        })
    }

    fn update(&mut self, ctx: &mut Context, delta: f32) {
        if self.input.process_updates {
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
            self.input
                .controller
                .update_camera(camera, transform, delta)
        });

        if self.input.draw_ui {
            self.ui.draw(ctx, &mut self.scene, delta);
        }

        self.scene.update(delta);
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        ctx.render()?
            .draw_bounds(self.input.draw_bounds)
            .draw_wire(self.input.draw_wire)
            .with_renderable(&self.scene, RenderType::Scene)?
            .build()
    }

    fn input(&mut self, ctx: &mut Context, input: Input) {
        self.input.input(ctx, input, self.ui.ui_hovered);

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

    fn resize(&mut self, _ctx: &mut Context, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    async fn start(&mut self, ctx: &mut Context) {
        self.init(ctx).await;
    }

    fn should_update(&mut self, _ctx: &mut Context) -> bool {
        self.input.process_updates
    }

    fn close(&mut self, _ctx: &mut Context) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl<Context: GobsContext> App<Context> {
    async fn init(&mut self, ctx: &mut Context) {
        ctx.load_material("materials.ron").await;

        let diffuse_texture = ctx
            .new_texture("Wall Diffuse")
            .diffuse(examples::WALL_TEXTURE, examples::DIFFUSE_FORMAT)
            .build();
        let normal_texture = ctx
            .new_texture("Wall Normal")
            .normal(examples::WALL_TEXTURE_N, examples::NORMAL_FORMAT)
            .build();

        let material = ctx
            .new_material("normal")
            .from_base("normal")
            .with_textures(&[diffuse_texture, normal_texture])
            .build();

        let mesh = ctx
            .new_mesh("cube")
            .with_geometry(Shapes::cubemap(1, 1, &[0], 1.))
            .for_material(material)
            .build();

        let cube = ctx
            .new_model("cube")
            .with_mesh(mesh)
            .with_material(material)
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

    Application::<App<GameContext>>::new("Scenegraph", examples::WIDTH, examples::HEIGHT).run();
}
