use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Key, Transform, logger},
    game::{AppError, Application, GameContext, GobsGame, context::GobsContext as _},
    render::{
        MaterialInstanceProperties, MaterialsConfig, Model, RenderError, RenderType, Shapes,
        TextureProperties, TextureType,
    },
    resource::ResourceLifetime,
    scene::{
        components::{NodeId, NodeValue},
        graph::scenegraph::SceneGraph,
        scene::Scene,
    },
};

use examples::{InputManager, Ui};

struct App {
    scene: Scene,
    ui: Ui,
    input: InputManager,
    nodes: Vec<NodeId>,
}

impl GobsGame<GameContext> for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
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

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
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

        self.scene.update(&ctx.renderer.gfx, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        ctx.render()?
            .draw_bounds(self.input.draw_bounds)
            .with_renderable(&self.scene, RenderType::Scene)?
            .build()
    }

    fn input(&mut self, _ctx: &mut GameContext, input: Input) {
        self.input.input(input, false);

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
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
    }

    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        self.input.process_updates
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &mut GameContext) {
        MaterialsConfig::load_resources("materials.ron", &mut ctx.resource_manager).await;

        let properties = TextureProperties::with_file(
            "Wall Diffuse",
            examples::DIFFUSE_FORMAT,
            examples::WALL_TEXTURE,
        );
        let diffuse_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static, false);

        let mut properties = TextureProperties::with_file(
            "Wall Normal",
            examples::NORMAL_FORMAT,
            examples::WALL_TEXTURE_N,
        );
        properties.format.ty = TextureType::Normal;
        let normal_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static, false);

        let material = ctx.resource_manager.get_by_name("normal").unwrap();

        let material_instance_properties = MaterialInstanceProperties::new("normal", material)
            .textures(&[diffuse_texture, normal_texture]);
        let material_instance = ctx.resource_manager.add(
            material_instance_properties,
            ResourceLifetime::Static,
            false,
        );

        let cube = Model::builder("cube")
            .mesh(
                Shapes::cubemap(1, 1, &[1], 1.),
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

    Application::<App, GameContext>::new("Scenegraph", examples::WIDTH, examples::HEIGHT).run();
}
