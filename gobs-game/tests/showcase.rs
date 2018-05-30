extern crate cgmath;
extern crate image;

extern crate gobs_game as game;
extern crate gobs_scene as scene;

use std::sync::Arc;
use std::fs::File;
use std::io::{BufRead, BufReader};

use cgmath::{Point3, Vector3};

use scene::SphericalCoord;
use scene::LightBuilder;
use scene::SceneGraph;
use scene::model::{Color, MeshInstanceBuilder};

use game::app::{Application, Run};
use game::asset::{AssetManager, TileMap};
use game::input::Key;
use game::timer::Timer;

pub enum Example {
    FONT = 0,
    FONTMAP,
    TILE,
    CHECK,
    MAP,
    DEPTH,
    CUBE,
    DUNGEON
}

struct App {
    graph: SceneGraph,
    uigraph: SceneGraph,
    timer: Timer,
    position: SphericalCoord<f32>,
    world_size: f32,
    selected: Example,
    show_fps: bool,
    show_centers: bool
}

impl Run for App {
    fn create(&mut self, engine: &mut Application) {
        self.draw_scene(engine);
    }

    fn update(&mut self, engine: &mut Application) {
        self.handle_input(engine);

        let batch = engine.batch_mut();

        batch.begin();

        batch.draw_graph(&self.graph);
        batch.draw_graph(&self.uigraph);

        batch.end();

        self.print_fps();
    }

    fn resize(&mut self, width: u32, height: u32, _engine: &mut Application) {
        let (world_width, world_height) = {
            let ratio = width as f32 / height as f32;
            let world_height = self.world_size;

            (world_height * ratio, world_height)
        };

        self.graph.camera_mut().resize(world_width, world_height);
    }
}

impl App {
    fn new() -> App {
        let position = SphericalCoord::new(3., 0., 0.);
        let mut graph = SceneGraph::new();
        let light = LightBuilder::new()
            .directional(Vector3::new(-1., -1., -1. ))
            .build();
        graph.set_light(light);

        App {
            graph: SceneGraph::new(),
            uigraph: SceneGraph::new(),
            timer: Timer::new(),
            position: position,
            world_size: 5.0,
            selected: Example::FONT,
            show_fps: false,
            show_centers: false
        }
    }

    fn move_camera(&mut self, dr: f32, dtheta: f32, dphi: f32) {
        self.position.update(dr, dtheta, dphi);

        self.update_camera();
    }

    fn goto(&mut self, r: f32, theta: f32, phi: f32) {
        self.position.set(r, theta, phi);

        self.update_camera();
    }

    fn update_camera(&mut self) {
        let pos: Point3<f32> = self.position.into();
        let dir = Vector3::new(-pos.x, -pos.y, -pos.z);
        let up = Vector3::new(0., 1., 0.);

        self.graph.camera_mut().set_position(pos);
        self.graph.camera_mut().look_at(dir, up);
    }

    fn ortho(&mut self, size: f32) {
        self.graph.camera_mut().set_ortho(-10., 10.);
        self.world_size = size;
        self.update_camera();
    }

    fn perspective(&mut self, size: f32) {
        self.graph.camera_mut().set_perspective(60., 0.1, 20.);
        self.world_size = size;
        self.update_camera();
    }

    fn print_fps(&mut self) {
        if self.show_fps {
            println!("{} ms", self.timer.delta() / 1000000);
        }
    }

    fn handle_input(&mut self, engine: &mut Application) {
        let mut redraw = false;

        {
            let input_map = engine.input_map();

            if input_map.pressed(Key::LEFT) {
                self.move_camera(0., -0.05, 0.);
            }

            if input_map.pressed(Key::RIGHT) {
                self.move_camera(0., 0.05, 0.);
            }

            if input_map.pressed(Key::UP) {
                self.move_camera(0., 0., 0.05);
            }

            if input_map.pressed(Key::DOWN) {
                self.move_camera(0., 0., -0.05);
            }

            if input_map.pressed(Key::PAGEUP) {
                self.move_camera(-0.05, 0., 0.);
            }

            if input_map.pressed(Key::PAGEDOWN) {
                self.move_camera(0.05, 0., 0.);
            }

            if input_map.pressed(Key::RETURN) {
                self.goto(3., 0., 0.);
            }

            if input_map.pressed(Key::SPACE) {
                self.next_scene();

                redraw = true;
            }

            if input_map.pressed(Key::TAB) {
                self.uigraph.clear();
                self.show_centers = !self.show_centers;
                if self.show_centers {
                    self.draw_centers();
                }
            }

            if input_map.pressed(Key::A) {
                let light = LightBuilder::new().directional(Vector3::new(-1., -1., -1. )).build();
                self.graph.set_light(light);
            }

            if input_map.pressed(Key::Z) {
                let light = LightBuilder::new().point(Point3::new(1., 0., 1.))
                    .color(Color::blue()).build();
                self.graph.set_light(light);
            }

            if input_map.pressed(Key::F) {
                self.show_fps = !self.show_fps;
                if self.show_fps {
                    self.timer.reset();
                }
            }
        }

        if redraw {
            self.draw_scene(engine);
        }
    }

    fn draw_scene(&mut self, engine: &mut Application) {
        self.graph.clear();

        {
            match self.selected {
                Example::FONT => self.draw_font(),
                Example::FONTMAP => self.draw_fontmap(),
                Example::TILE => self.draw_tile(),
                Example::CHECK => self.draw_checkboard(),
                Example::MAP => self.draw_map(),
                Example::DEPTH => self.draw_depth(),
                Example::CUBE => self.draw_cube(),
                Example::DUNGEON => self.draw_dungeon()
            }
        }

        let dim = engine.dimensions();
        self.resize(dim[0], dim[1], engine)
    }

    fn next_scene(&mut self) {
        self.selected = match self.selected {
            Example::FONT => Example::FONTMAP,
            Example::FONTMAP => Example::TILE,
            Example::TILE => Example::CHECK,
            Example::CHECK => Example::MAP,
            Example::MAP => Example::DEPTH,
            Example::DEPTH => Example::CUBE,
            Example::CUBE => Example::DUNGEON,
            Example::DUNGEON => Example::FONT
        }
    }

    fn asset(filename: &str) -> String {
        format!("../../assets/{}", filename)

    }

    fn draw_map(&mut self) {
        self.ortho(40.);

        let texture = AssetManager::get_color_texture(Color::white());

        let mesh = AssetManager::build_quad();

        let f = File::open(Self::asset("dungeon.map")).expect("File not found");
        let reader = BufReader::new(f);

        for (num, line) in reader.lines().enumerate() {
            for (col, c) in line.unwrap().chars().enumerate() {
                match c {
                    'w' => {
                        let (x, y) = (col as f32, num as f32);

                        let tile = MeshInstanceBuilder::new(mesh.clone())
                            .color(Color::red())
                            .texture(texture.clone())
                            .translate((x - 16., 16. - y, 0.0))
                            .build();
                        self.graph.add_instance(Arc::new(tile));
                    },
                    _ => ()
                }
            }
        }
    }

    fn draw_checkboard(&mut self) {
        self.ortho(30.);

        let texture = AssetManager::get_color_texture(Color::white());

        let triangle = AssetManager::build_triangle();
        let square = AssetManager::build_quad();

        for i in -5..5 {
            for j in -5..5 {
                let tile = match i + j {
                    k if (k % 2 == 0) => {
                        let color = Color::red();

                        MeshInstanceBuilder::new(triangle.clone())
                            .color(color)
                            .texture(texture.clone())
                            .translate((i as f32, j as f32, 0.0))
                            .build()
                    },
                    _ => {
                        let color = Color::white();

                        MeshInstanceBuilder::new(square.clone())
                            .color(color)
                            .texture(texture.clone())
                            .translate((i as f32, j as f32, 0.0))
                            .build()
                    },
                };

                self.graph.add_instance(Arc::new(tile));
            }
        }
    }

    fn draw_tile(&mut self) {
        self.ortho(5.);

        let tilemap = {
            let tile_size = [34, 34];

            let texture = AssetManager::load_texture(&Self::asset("tileset.png"));

            TileMap::new(texture, tile_size)
        };

        let mut tile = tilemap.build_tile(0, 0);
        tile.translate((-1.0, 1.0, 0.0));
        self.graph.add_instance(Arc::new(tile));

        let mut tile = tilemap.build_tile(0, 20);
        tile.translate((-1.0, -1.0, 0.0));
        self.graph.add_instance(Arc::new(tile));

        let mut tile = tilemap.build_tile(0, 21);
        tile.translate((1.0, 1.0, 0.0));
        self.graph.add_instance(Arc::new(tile));

        let mut tile = tilemap.build_tile(0, 22);
        tile.translate((1.0, -1.0, 0.0));
        self.graph.add_instance(Arc::new(tile));
    }

    fn draw_cube(&mut self) {
        self.perspective(30.);

        let texture = AssetManager::load_texture(&Self::asset("wall.png"));

        let mesh = AssetManager::build_cube();

        let instance = MeshInstanceBuilder::new(mesh.clone())
            .color(Color::white())
            .texture(texture)
            .build();

        self.graph.add_instance(Arc::new(instance));
    }

    fn draw_dungeon(&mut self) {
        self.perspective(30.);

        let texture = AssetManager::load_texture(&Self::asset("wall.png"));

        let mesh = AssetManager::build_cube();

        let f = File::open(Self::asset("dungeon.map")).expect("File not found");
        let reader = BufReader::new(f);

        for (num, line) in reader.lines().enumerate() {
            for (col, c) in line.unwrap().chars().enumerate() {
                match c {
                    'w' => {
                        let (x, y) = (col as f32, num as f32);

                        let instance = MeshInstanceBuilder::new(mesh.clone())
                            .color(Color::white())
                            .texture(texture.clone())
                            .translate((x - 16., 0., y - 16.))
                            .build();

                        self.graph.add_instance(Arc::new(instance));
                    },
                    _ => ()
                }
            }
        }

        println!("{}", self.graph.instances().len());
    }

    fn draw_depth(&mut self) {
        self.ortho(30.);

        let texture = AssetManager::get_color_texture(Color::white());

        let triangle = AssetManager::build_triangle();

        for i in -10..11 {
            let color = match i {
                -10 | 10 => Color::red(),
                0 => Color::yellow(),
                _ => Color::green()
            };

            let i = i as f32;

            let instance = MeshInstanceBuilder::new(triangle.clone())
                .color(color)
                .texture(texture.clone())
                .translate((i, 0., i / 10.))
                .build();

            self.graph.add_instance(Arc::new(instance));
        }
    }

    fn draw_font(&mut self) {
        self.ortho(1.);

        let size: usize = 30;

        let font = AssetManager::load_font(size, &Self::asset("font.ttf"));

        let chars = font.layout("Press space to go to the new example");

        for c in chars {
            self.graph.add_instance(Arc::new(c));
        }

    }

    fn draw_fontmap(&mut self) {
        self.ortho(30.);

        let size: usize = 100;

        let font = AssetManager::load_font(size, &Self::asset("font.ttf"));
        let mesh = AssetManager::build_quad();

        let mut text = MeshInstanceBuilder::new(mesh.clone())
            .texture(font.texture())
            .scale(10., 10., 1.)
            .build();


        self.graph.add_instance(Arc::new(text));
    }

    fn draw_centers(&mut self) {
        self.ortho(2.);

        let texture = AssetManager::get_color_texture(Color::green());

        let left: Point3<f32> = [-1., 0., 0.5].into();
        let right: Point3<f32> = [1., 0., 0.5].into();
        let top: Point3<f32> = [0., 1., 0.5].into();
        let bottom: Point3<f32> = [0., -1., 0.5].into();

        let line = AssetManager::build_line(left, right);
        let instance = MeshInstanceBuilder::new(line).texture(texture.clone()).build();
        self.uigraph.add_instance(Arc::new(instance));

        let line = AssetManager::build_line(bottom, top);
        let instance = MeshInstanceBuilder::new(line).texture(texture).build();
        self.uigraph.add_instance(Arc::new(instance));
    }
}

#[test]
pub fn showcase() {
    let mut engine = Application::new();
    engine.run(App::new());
}
