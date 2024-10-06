#![feature(portable_simd)]

use std::{
    collections::{HashSet, VecDeque},
};

use koi3::*;
use koi_graphics_context::{FilterMode, TextureSettings};

fn main() {
    App::default()
        .with_resource(InitialSettings {
            color_space: koi_graphics_context::ColorSpace::SRGB,
            window_height: 1000,
            window_width: 1400,
            ..Default::default()
        })
        .setup_and_run(|world, resources| {
            let camera_min = 60.0;
            let camera_max = 140.0;

            let world_size = Vec2::new(90.0, 200.0);

            let camera = world.spawn((Transform::new().with_position(Vec3::new(
                world_size.x / 2.0,
                camera_max,
                0.0,
            )),));

            let camera_child = world.spawn((
                Transform::new(),
                Camera {
                    projection_mode: ProjectionMode::Orthographic {
                        height: 80.0, //60.0,
                        z_near: 100.0,
                        z_far: -100.0,
                    },
                    clear_color: Some(Color::BLACK),
                    ..Default::default()
                },
            ));
            let _ = world.set_parent(camera, camera_child);

            let grid_world_position = world_size.extend(0.0) / 2.0;
            let grid_world_display = world.spawn((
                Transform::new()
                    .with_scale(world_size.extend(1.0))
                    .with_position(world_size.extend(30.0) / 2.0),
                Mesh::VERTICAL_QUAD,
            ));

            let mut grid_world = GridWorld::new(world_size.x as _, world_size.y as _);
            grid_world.initial_terrain_setup();
            grid_world.reverse_deterent_pheromones();

            let ant_material = get_texture_material(
                "assets/AntFrame1.png",
                resources,
                Shader::UNLIT,
                Color::BLACK,
            );

            let spider_material = get_texture_material(
                "assets/spider.png",
                resources,
                Shader::UNLIT,
                Color::RED,
            );

            let a_material = ant_material.clone();
            let spawn_ants = move |world: &mut World, ant_count: usize| {
                let mut random = Random::new();

                for i in 0..ant_count {
                    world.spawn((
                        Transform::new()
                            .with_position(
                                Vec2::new(
                                    (i as f32 / ant_count as f32) * world_size.x,
                                    world_size.y - 5.0,
                                )
                                .extend(-3.0),
                            )
                            .with_scale(Vec3::fill(2.0)),
                        Ant {
                            dir: -Vec2::Y,
                            speed: random.range_f32(0.05..0.2),
                            return_mode: false,
                            health: 255,
                            carrying: None,
                            home: Vec2::ZERO,
                            is_spider: false,
                        },
                        Mesh::VERTICAL_QUAD,
                        //Material::UNLIT,
                        a_material.clone(),
                    ));
                }
            };
            // Setup a few ants for the first screen.
            spawn_ants(world, 10);

            let foot_material = get_texture_material(
                "assets/foot.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );

            let mut sandbox_mode = false;
            {
                world.spawn((
                    Transform::new()
                        .with_position((Vec2::new(40.0, 30.0)).extend(-1.0))
                        .with_scale(Vec3::fill(50.0)),
                    Mesh::VERTICAL_QUAD,
                    foot_material.clone(),
                ));

                world.spawn((
                    Transform::new()
                        .with_position((Vec2::new(60.0, 30.0)).extend(-1.0))
                        .with_scale(Vec3::fill(50.0))
                        .with_rotation(Quaternion::from_angle_axis(std::f32::consts::PI, Vec3::Y)),
                    Mesh::VERTICAL_QUAD,
                    foot_material.clone(),
                ));
            }

            
            let squish_sounds = {
                let mut sounds = resources
                .get::<AssetStore<Sound>>();
                [
                   sounds.load("assets/squish0.wav", SoundSettings{ scale: 0.5}),
                   sounds.load("assets/squish2.wav", SoundSettings{ scale: 0.5}),
                   sounds.load("assets/squish3.wav", SoundSettings{ scale: 0.5}),
                ]
            };

            let suck_sound = {
                let mut sounds = resources
                .get::<AssetStore<Sound>>();
                    sounds.load("assets/suck.wav", SoundSettings{ scale: 0.5})
            };

            let spit_sounds = {
                let mut sounds = resources
                .get::<AssetStore<Sound>>();
                [
                   sounds.load("assets/spit0.wav", SoundSettings{ scale: 0.5}),
                   sounds.load("assets/spit1.wav", SoundSettings{ scale: 0.5}),
                ]
            };

            let gun_sounds = {
                let mut sounds = resources
                .get::<AssetStore<Sound>>();
                [
                   sounds.load("assets/gun0.wav", Default::default()),
                   //sounds.load("assets/gun1.wav", Default::default()),
                ]
            };

            let defeat_sounds = {
                let mut sounds = resources
                .get::<AssetStore<Sound>>();
                [
                   sounds.load("assets/defeat.wav", Default::default()),
                ]
            };

            let out_of_bullets_sounds = {
                let mut sounds = resources
                .get::<AssetStore<Sound>>();
                [
                   sounds.load("assets/out_of_bullets.wav", {
                    SoundSettings {
                        scale: 0.05
                    }
                   }),
                   //sounds.load("assets/gun1.wav", Default::default()),
                ]
            };

            let strawhand = get_texture_material(
                "assets/strawhand.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );


            let pointed_finger_material = get_texture_material(
                "assets/pointedfinger.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );
            let spiderhand = get_texture_material(
                "assets/spiderhand.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );

            let curled_finger_material = get_texture_material(
                "assets/curledfinger.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );

            let toothpaste_hand = get_texture_material(
                "assets/toothpastehand.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );

            let gameover = get_texture_material(
                "assets/gameover.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );

            let victory = get_texture_material(
                "assets/victoryscreen.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );

            let titlescreen = get_texture_material(
                "assets/titlescreen.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );

            let pointed_gun = get_texture_material(
                "assets/pointedgun.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );

            let gun_fire = get_texture_material(
                "assets/gunfire.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );

            let reticule = get_texture_material(
                "assets/reticule.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::RED,
            );

            let nacho_hand = get_texture_material(
                "assets/nachohand.png",
                resources,
                Shader::UNLIT_TRANSPARENT,
                Color::WHITE,
            );

            let reticule_entity = world.spawn((
                Transform::new()
                    .with_position(Vec2::new(22.0, -22.0).extend(-20.0))
                    .with_scale(Vec3::fill(4.0)),
                reticule,
                RenderFlags::DEFAULT,
                //Material::UNLIT,
                Mesh::VERTICAL_QUAD,
            ));

            let gameoverscreen = world.spawn((
                Transform::new()
                    .with_scale(Vec3::fill(85.0))
                    .with_position(Vec3::Z * -2.9),
                RenderFlags::NONE,
                gameover,
                Mesh::VERTICAL_QUAD,
            ));
            let _ = world.set_parent(camera, gameoverscreen);

            let victoryscreen = world.spawn((
                Transform::new()
                    .with_scale(Vec3::fill(85.0))
                    .with_position(Vec3::Z * -2.9),
                RenderFlags::NONE,
                victory,
                Mesh::VERTICAL_QUAD,
            ));
            let _ = world.set_parent(camera, victoryscreen);

            let titlescreen = world.spawn((
                Transform::new()
                    .with_scale(Vec3::fill(85.0))
                    .with_position(Vec3::Z * -2.9),
                RenderFlags::DEFAULT,
                titlescreen,
                Mesh::VERTICAL_QUAD,
            ));
            let _ = world.set_parent(camera, titlescreen);

            let player_item = world.spawn((Transform::new(),));
            let player_item_art = world.spawn((
                Transform::new()
                    .with_position(Vec2::new(22.0, -22.0).extend(-20.0))
                    .with_scale(Vec3::fill(50.0)),
                curled_finger_material.clone(),
                RenderFlags::DEFAULT,
                //Material::UNLIT,
                Mesh::VERTICAL_QUAD,
            ));
            let _ = world.set_parent(player_item, player_item_art);

            let mut current_item = CurrentItem::Finger;

            let mut pointer_last_position = Vec3::ZERO;

            let mut started_once = false;

            struct Ant {
                dir: Vec2,
                speed: f32,
                return_mode: bool,
                health: u8,
                carrying: Option<TileMaterial>,
                is_spider: bool,
                home: Vec2,
            }

            let a_material = ant_material.clone();
            let spawn_wave = move |world: &mut World, wave: &Wave| {
                let mut random = Random::new();

                for i in 0..wave.ant_count {
                    world.spawn((
                        Transform::new()
                            .with_position(
                                Vec2::new(
                                    (i as f32 / wave.ant_count as f32) * world_size.x,
                                    world_size.y - 5.0,
                                )
                                .extend(-3.0),
                            )
                            .with_scale(Vec3::fill(2.0)),
                        Ant {
                            dir: -Vec2::Y,
                            speed: random.range_f32(0.05..0.2),
                            return_mode: false,
                            health: 255,
                            carrying: None,
                            is_spider: false,
                            home: Vec2::ZERO
                        },
                        Mesh::VERTICAL_QUAD,
                        //Material::UNLIT,
                        a_material.clone(),
                    ));
                }
            };

            let mut random = Random::new();

            // grid_world.update_texture(world, resources, grid_world_display);

            let mut skip_first = true;
            let mut toothpaste_squeeze = 2.0;

            let mut pointer_position = Vec3::ZERO;
            let mut pointer_max_speed = None;

            let player_health_max = 25;
            let mut player_health = player_health_max;

            let mut screen_shake_amount = 0.0;

            // Also used for a heal effect
            let mut player_hurt_effect: f32 = 0.0;

            let mut intro_interpolate = 0.0;

            let gun_cooldown_animation_reset = 1.0;
            let mut gun_cooldown_animation = 0.0;

            struct UIState {
                ui_lines: Vec<String>,
                transparency: f32,
                display_bonus_text: f32,
                bonus_text: String,
            }

            let slider_material = resources.get::<AssetStore<Material>>().add(Material {
                base_color: Color::from_srgb_hex(0xdbcccc, 1.0),
                shader: Shader::UNLIT,
                ..Default::default()
            });

            let gun_slider_material = resources.get::<AssetStore<Material>>().add(Material {
                base_color: Color::YELLOW,
                shader: Shader::UNLIT,
                ..Default::default()
            });

            let bullets_slider_material = resources.get::<AssetStore<Material>>().add(Material {
                base_color: Color::from_srgb_hex(0xeb1400, 1.0),
                shader: Shader::UNLIT,
                ..Default::default()
            });

            let nacho_slider_material = resources.get::<AssetStore<Material>>().add(Material {
                base_color: Color::ORANGE.with_lightness(0.7),
                shader: Shader::UNLIT,
                ..Default::default()
            });

            let toothpaste_slider_color = resources.get::<AssetStore<Material>>().add(Material {
                base_color: Color::from_srgb_hex(0x2589D0, 1.0),
                shader: Shader::UNLIT,
                ..Default::default()
            });

            #[derive(Clone, Copy, Debug)]
            enum SliderOption {
                Gun,
                Bullets,
                Nachos,
                Toothpaste,
            }

            struct Slider {
                line_position: f32,
                line_dir: f32,
                line_entity: Entity,
                targets: Vec<((f32, f32), SliderOption, Entity)>,
                entities: Vec<Entity>,
                visible: bool,
                lost_entries: Vec<(f32, SliderOption, Handle<Material>)>,
            }

            fn ranges_overlap(a: (f32, f32), b: (f32, f32)) -> bool {
                let (a_start, a_end) = a;
                let (b_start, b_end) = b;

                // Check if the ranges overlap
                a_start <= b_end && b_start <= a_end
            }

            impl Slider {
                pub fn add_option(
                    &mut self,
                    world: &mut World,
                    len: f32,
                    option: SliderOption,
                    material: Handle<Material>,
                ) {
                    let parent = self.entities[0];

                    let mut r = Random::new();

                    let mut start = None;
                    // Try 20 times to insert
                    for _ in 0..20 {
                        let mut overlaps = false;
                        let proposed_start = r.f32();

                        if proposed_start + len > 1.0 {
                            continue;
                        }

                        for t in self.targets.iter() {
                            let overlaps_here =
                                ranges_overlap((proposed_start, proposed_start + len), t.0);
                            if overlaps_here {
                                overlaps = true;
                                break;
                            }
                        }

                        if !overlaps {
                            start = Some(proposed_start);
                            break;
                        }
                    }
                    

                    if let Some(start) = start {
                        let e = world.spawn((
                            Transform::new()
                                .with_position(Vec3::new(-0.5 + start + len / 2.0, -0.0, -1.0))
                                .with_scale(Vec3::new(len, 1.0, 1.0)),
                            Mesh::VERTICAL_QUAD,
                            if self.visible {
                                RenderFlags::DEFAULT
                            } else {
                                RenderFlags::NONE
                            },
                            material.clone(),
                        ));
                        let _ = world.set_parent(parent, e);

                        self.targets.push(((start, start + len), option, e));
                    } else {
                        self.lost_entries.push((len, option, material));
                    }
                }
                pub fn new(
                    world: &mut World,
                    parent: Entity,
                    base_material: Handle<Material>,
                ) -> Self {
                    let mut entities = Vec::new();
                    let base_entity = world.spawn((
                        Transform::new()
                            .with_position(Vec3::new(0.0, -20.0, -30.0))
                            .with_scale(Vec3::new(50.0, 5.0, 1.0)),
                        Mesh::VERTICAL_QUAD,
                        RenderFlags::DEFAULT,
                        base_material,
                    ));
                    let _ = world.set_parent(parent, base_entity);
                    entities.push(base_entity);

                    let line_entity = world.spawn((
                        Transform::new()
                            .with_position(Vec3::new(0.0, 0.0, -2.0))
                            .with_scale(Vec3::new(0.01, 1.4, 1.0)),
                        Mesh::VERTICAL_QUAD,
                        Material::UNLIT,
                        RenderFlags::DEFAULT,
                    ));
                    let _ = world.set_parent(base_entity, line_entity);
                    entities.push(line_entity);

                    let mut s = Self {
                        line_position: 0.0,
                        line_dir: 1.0,
                        line_entity,
                        targets: Vec::new(),
                        entities,
                        visible: true,
                        lost_entries: Vec::new(),
                    };
                    s.move_line(world);
                    s
                }

                fn move_line(&mut self, world: &mut World) {
                    let mut e = world.get::<&mut Transform>(self.line_entity).unwrap();
                    e.position.x = -0.5 + self.line_position;
                }

                pub fn progress_line(&mut self, world: &mut World) {
                    self.line_position += self.line_dir * 0.01;

                    if self.line_position > 1.0 {
                        self.line_position = 1.0;
                        self.line_dir = -1.0;
                    }

                    if self.line_position <= 0.0 {
                        self.line_position = 0.0;
                        self.line_dir = 1.0;
                    }

                    self.move_line(world);
                }

                pub fn insert_random_lost_entry(&mut self, world: &mut World) {
                    if !self.lost_entries.is_empty() {
                        let i = Random::new().range_u32(0..self.lost_entries.len() as _);
                        let removed = self.lost_entries.remove(i as _);
                        self.add_option(world, removed.0, removed.1, removed.2);
                    }
                }

                pub fn remove_random(&mut self, world: &mut World) {
                    if !self.targets.is_empty() {
                        let i = Random::new().range_u32(0..self.targets.len() as _);
                        let removed: ((f32, f32), SliderOption, Entity) =
                            self.targets.remove(i as _);
                        let material =
                            (*world.get::<&Handle<Material>>(removed.2).unwrap()).clone();

                        let _ = world.despawn(removed.2);
                        self.lost_entries
                            .push((removed.0 .1 - removed.0 .0, removed.1, material));
                    }
                }

                pub fn stop_line(&mut self, world: &mut World) -> Option<SliderOption> {
                    if !self.visible {
                        return None;
                    }

                    let mut found_target = None;

                    for (i, t) in self.targets.iter().enumerate() {
                        let overlaps_here = ranges_overlap(
                            (self.line_position - 0.02, self.line_position + 0.02),
                            t.0,
                        );

                        if overlaps_here {
                            found_target = Some(i);
                            break;
                        }
                    }

                    if let Some(found_target) = found_target {
                        let removed = self.targets.remove(found_target);
                        let _ = world.despawn(removed.2);
                        Some(removed.1)
                    } else {
                        None
                    }
                }

                pub fn hide(&mut self, world: &mut World) {
                    self.visible = false;
                    for entity in self.entities.iter() {
                        *world.get::<&mut RenderFlags>(*entity).unwrap() = RenderFlags::NONE;
                    }

                    for (_, _, e) in self.targets.iter() {
                        *world.get::<&mut RenderFlags>(*e).unwrap() = RenderFlags::NONE;
                    }
                }

                pub fn show(&mut self, world: &mut World) {
                    self.visible = true;
                    for entity in self.entities.iter() {
                        *world.get::<&mut RenderFlags>(*entity).unwrap() = RenderFlags::DEFAULT;
                    }

                    for (_, _, e) in self.targets.iter() {
                        *world.get::<&mut RenderFlags>(*e).unwrap() = RenderFlags::DEFAULT;
                    }
                }

                pub fn clear(&mut self, world: &mut World) {
                    for (_, _, e) in self.targets.drain(..) {
                        let _ = world.despawn(e);
                    }
                    self.lost_entries.clear();
                }
            }

            let mut slider = Slider::new(world, camera, slider_material);
            slider.hide(world);

            {
                use kui::*;
                let style = StandardStyle::default();

                let mut fonts = Fonts::empty();
                fonts
                    .new_font_from_bytes(include_bytes!("../assets/KiteOne-Regular.ttf"))
                    .unwrap();

                fonts
                    .new_font_from_bytes(include_bytes!("../assets/Lekton-Bold.ttf"))
                    .unwrap();

                let first_child = center(max_width(
                    700.0,
                    column({
                        let t0 = text(|state: &mut UIState| {
                            state.ui_lines.get(0).cloned().unwrap_or_default()
                        })
                        .with_size(|_, _, _| 40.0)
                        .with_color(|state, _, _| {
                            let mut transparency = state.transparency - 0.01;
                            if state.transparency > 1.5 {
                                transparency = 0.0
                            }
                            Color::BLACK.with_alpha(transparency)
                        });
                        let t1 = text(|state: &mut UIState| {
                            state.ui_lines.get(1).cloned().unwrap_or_default()
                        })
                        .with_size(|_, _, _| 40.0)
                        .with_color(|state, _, _| {
                            let mut transparency =
                                ((state.transparency - 0.5) / 1.0).clamp(0.0, 1.0);

                            if state.transparency > 2.0 {
                                transparency = 0.0
                            }
                            Color::BLACK.with_alpha(transparency)
                        });

                        let t3 = text(|state: &mut UIState| {
                            state.ui_lines.get(2).cloned().unwrap_or_default()
                        })
                        .with_size(|_, _, _| 40.0)
                        .with_color(|state, _, _| {
                            let mut transparency =
                                ((state.transparency - 0.9) / 1.0).clamp(0.0, 1.0);

                            if state.transparency > 2.0 {
                                transparency = 0.0
                            }
                            Color::BLACK.with_alpha(transparency)
                        });

                        (t0, t1, t3)
                    }),
                ));
                let second_child = center(
                    text(|s: &mut UIState| {
                        let mut v = String::from("\n\n\n\n");
                        v.push_str(&s.bonus_text);
                        v
                    })
                    .with_color(|_: &mut UIState, _, _| Color::YELLOW)
                    .with_font(|_, _, _| Font::from_index(1))
                    .with_size(|_, _, _| 20.0),
                );
                // expand(fill(|_, _, _| Color::RED.with_alpha(0.5)))
                ScreenSpaceUI::<UIState>::new(
                    world,
                    resources,
                    style,
                    fonts,
                    None,
                    None,
                    stack((first_child, second_child)),
                );
            }

            resources.add(UIState {
                ui_lines: Vec::new(),
                transparency: 1.0,
                display_bonus_text: 0.0,
                bonus_text: String::new(),
            });

            enum WaveTrigger {
                LowAntCount(usize),
                FramesElapsedSinceLast(u32),
            }

            struct Wave {
                victory_condition: WaveTrigger,
                frame_delay: usize,
                ant_count: usize,
                ui_lines: Vec<String>,
                bonus_text: String,
                rummage_options: Vec<SliderOption>,
            }

            impl Default for Wave {
                fn default() -> Self {
                    Self {
                        victory_condition: WaveTrigger::LowAntCount(0),
                        frame_delay: 0,
                        ant_count: 10,
                        ui_lines: Vec::new(),
                        rummage_options: Vec::new(),
                        bonus_text: String::new(),
                    }
                }
            }

            let mut current_wave: i32 = -1;
            let waves = vec![
                Wave {
                    victory_condition: WaveTrigger::LowAntCount(0),
                    frame_delay: 0,
                    ant_count: 5,
                    ui_lines: vec![
                        "\"The man who retreats...".into(),
                        "...is no longer a man\"".into(),
                    ],
                    rummage_options: vec![
                        SliderOption::Gun,
                    ],
                    ..Default::default()
                },
                Wave {
                    victory_condition: WaveTrigger::LowAntCount(0),
                    frame_delay: 0,
                    ant_count: 10,
                    ui_lines: vec!["WAVE 2".into(), "Press 2 to use the straw".into(), "Click to fire. Hold to suck.".into()],
                    rummage_options: vec![
                        SliderOption::Toothpaste,
                        SliderOption::Nachos,
                        SliderOption::Nachos,
                        SliderOption::Nachos,
                        SliderOption::Nachos,
                    ],
                    ..Default::default()
                },
                Wave {
                    victory_condition: WaveTrigger::LowAntCount(0),
                    frame_delay: 60 * 1,
                    ant_count: 20,
                    ui_lines: vec!["WAVE 3".into(), "Hold spacebar periodically to rummage in your pockets".into()],
                    rummage_options: vec![
                        SliderOption::Gun,
                        SliderOption::Toothpaste,
                        SliderOption::Nachos,
                        SliderOption::Nachos,
                        SliderOption::Nachos,
                        SliderOption::Nachos,
                        SliderOption::Nachos,
                    ],
                    ..Default::default()
                },
                Wave {
                    victory_condition: WaveTrigger::LowAntCount(0),
                    frame_delay: 60 * 5,
                    ant_count: 40,
                    ui_lines: vec![
                        "WAVE 4".into(),
                        "Use the straw to reach far off targets".into(),
                        "\"It is not the mountain we conquer\n but ourselves.\"".into(),
                    ],
                    rummage_options: vec![
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                        SliderOption::Nachos,
                    ],
                    ..Default::default()
                },
                Wave {
                    victory_condition: WaveTrigger::LowAntCount(0),
                    frame_delay: 0,
                    ant_count: 500,
                    ui_lines: vec![
                        "WAVE 5".into(),
                        "\"No man is more unhappy than he who never faces adversity.".into(),
                        "For he is not permitted to prove himself.\"".into(),
                    ],
                    rummage_options: vec![
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                        SliderOption::Toothpaste,
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                    ],
                    ..Default::default()
                },
                Wave {
                    victory_condition: WaveTrigger::LowAntCount(0),
                    frame_delay: 0,
                    ant_count: 10,
                    ui_lines: vec![
                        "The next wave will by your final battle.\n It will test you. Good luck".into(),
                    ],
                    rummage_options: vec![
                        SliderOption::Toothpaste,
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                    ],
                    ..Default::default()
                },
                Wave {
                    victory_condition: WaveTrigger::LowAntCount(0),
                    frame_delay: 0,
                    ant_count: 2000,
                    ui_lines: vec![
                        "WAVE 6: THE FINAL BATTLE".into(),
                        "\"Masculinity is not something given to you but something you gain.\n".into(),
                        "And you gain it by winning small battles with honor.\"".into(),
                    ],
                    rummage_options: vec![
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                        SliderOption::Bullets,
                    ],
                    ..Default::default()
                },
                Wave {
                    victory_condition: WaveTrigger::LowAntCount(0),
                    frame_delay: 0,
                    ant_count: 1,
                    ui_lines: vec![
                        "the very last ant...".into(),
                    ],
                    rummage_options: vec![
                      
                    ],
                    ..Default::default()
                },
            ];

            let mut frames_elapsed_since_last_wave = 0;

            let mut ui_transparency_animation = 0.0;

            let mut spawn_wave_override = false;

            let mut wave_text_shown = false;

            let mut bullet_count = 0;
            let mut nacho_crumb_count: i32 = 0;
            let mut toothpaste_tile_count: i32 = 0;
            let mut spider_count = 0;

            let mut rummage_count = 0;

            let mut wave_move_on_timer = 0;

            let mut inventory = HashSet::new();
            // inventory.insert(CurrentItem::Nacho);
            inventory.insert(CurrentItem::Straw);
            //inventory.insert(CurrentItem::Gun);


            let mut victory = false;
            let mut defeated = false;

            let  rummage_refresh_reset = 60 * 20;
            let mut rummage_refresh_timer = 0;

            let mut straw_contents: VecDeque<TileMaterial> = VecDeque::new();
            let mut straw_cooldown_timer = 0.0;
            let mut straw_sucking = false;

            struct SpitBall {
                target: Vec2,
                material: Vec<TileMaterial>
            }

            let mut current_straw_sound = None;

            let mut pointer_held_len = 0;
            

            move |event, world, resources| {
                if update_ui_with_event::<UIState>(world, resources, event) {
                    return;
                }
                match event {
                    Event::Draw => {
                        {
                            if victory {
                                *world.get::<&mut RenderFlags>(victoryscreen).unwrap() =
                                    RenderFlags::DEFAULT;

                                *world.get::<&mut RenderFlags>(player_item_art).unwrap() =
                                    RenderFlags::NONE;
                                world.get::<&mut Camera>(camera_child).unwrap().clear_color =
                                    Some(Color::BLACK);

                                let mut ui_state: std::sync::RwLockWriteGuard<'_, UIState> =
                                    resources.get::<UIState>();
                                ui_state.ui_lines.clear();
                                ui_state.bonus_text = String::new();
                            }

                            if started_once {
                                *world.get::<&mut RenderFlags>(titlescreen).unwrap() =
                                    RenderFlags::NONE;
                            }

                            let material_handle =
                                world.get::<&mut Handle<Material>>(player_item_art).unwrap();

                            let mut materials = resources.get::<AssetStore<Material>>();
                            let material = materials.get_mut(&material_handle);

                            let health_lerp =
                                (1.0 - (player_health as f32 / player_health_max as f32)).clamp(0.0, 1.0);
                            let mut color = Color::interpolate(
                                Color::WHITE,
                                Color::RED,
                                health_lerp * 3.0,
                            );

                            if player_hurt_effect < 0.0 {
                                color = Color::interpolate(
                                    color,
                                    Color::GREEN,
                                    player_hurt_effect.abs() / 1.0,
                                );
                            } else {
                                color = Color::interpolate(
                                    color,
                                    Color::RED,
                                    player_hurt_effect.abs() / 1.0,
                                );
                            };

                            if player_health > player_health_max {
                                color = Color::interpolate(
                                    color,
                                    Color::GREEN,
                                    ((player_health as f32 - player_health_max as f32) / 50.0).clamp(0.0, 1.0),
                                );
                            }

                            material.base_color = color;

                            let foot_material = materials.get_mut(&foot_material);
                            foot_material.base_color = color;

                            let ant_material = materials.get_mut(&ant_material);

                            if !victory && !sandbox_mode && player_health <= 0 {
                                if !defeated {
                                    let mut audio_manager: std::sync::RwLockWriteGuard<'_, AudioManager> = resources.get::<AudioManager>();
                                    let sounds = resources.get::<AssetStore<Sound>>();
                                    let sound = sounds.get(random.select_from_slice(&defeat_sounds));
                                    audio_manager.play_one_shot_with_speed(sound, random.range_f32(0.8..1.2));
                                    defeated = true;
                                }

                                ant_material.base_color = Color::WHITE;
                                *world.get::<&mut RenderFlags>(gameoverscreen).unwrap() =
                                    RenderFlags::DEFAULT;

                                *world.get::<&mut RenderFlags>(player_item_art).unwrap() =
                                    RenderFlags::NONE;
                                world.get::<&mut Camera>(camera_child).unwrap().clear_color =
                                    Some(Color::BLACK);

                                let mut ui_state: std::sync::RwLockWriteGuard<'_, UIState> =
                                    resources.get::<UIState>();
                                ui_state.bonus_text = String::new();

                            } else {
                                ant_material.base_color = Color::BLACK;
                                *world.get::<&mut RenderFlags>(gameoverscreen).unwrap() =
                                    RenderFlags::NONE;
                                *world.get::<&mut RenderFlags>(player_item_art).unwrap() =
                                    RenderFlags::DEFAULT;
                                world.get::<&mut Camera>(camera_child).unwrap().clear_color =
                                    Some(Color::BLACK);
                            }
                        }

                        grid_world.update_texture(world, resources, grid_world_display);

                        let screen_shake = Vec2::new(
                            random.range_f32(-screen_shake_amount..screen_shake_amount),
                            random.range_f32(-screen_shake_amount..screen_shake_amount),
                        );

                        let mut camera_transform =
                            world.get::<&mut Transform>(camera_child).unwrap();
                        camera_transform.position =
                            screen_shake.extend(camera_transform.position.z);
                        screen_shake_amount *= 0.90;
                        player_hurt_effect *= 0.94;
                    }
                    Event::KappEvent(KappEvent::KeyDown { key: Key::R, .. }) => {
                        screen_shake_amount += 3.0;
                        player_health -= 1;
                        println!("HEALTH: {:?}", player_health);
                        grid_world.reset_pheromones();
                    }
                    Event::KappEvent(KappEvent::KeyDown { key: Key::K, .. }) => {
                        // Enter sandbox mode
                        if !started_once {
                            inventory.insert(CurrentItem::Gun);
                            inventory.insert(CurrentItem::Toothpaste);
                            inventory.insert(CurrentItem::Straw);

                            bullet_count = i32::MAX;
                            nacho_crumb_count = i32::MAX;
                            sandbox_mode = true;
                            started_once = true;
                            grid_world.neutral_pheremones();
                            spawn_ants(world, 1);
                            intro_interpolate = 100.0;
                            toothpaste_tile_count = i32::MAX;
                            spider_count = i32::MAX;

                            let mut ui_state = resources.get::<UIState>();
                            ui_state.display_bonus_text = 200.0;
                            ui_state.bonus_text = "Welcome to SANDBOX MODE".into();
                        }
                    }
                    Event::KappEvent(KappEvent::KeyDown {
                        key: Key::Space, ..
                    })
                    | Event::KappEvent(KappEvent::KeyUp {
                        key: Key::Space, ..
                    }) => {
                        let key_up = match event {
                            Event::KappEvent(KappEvent::KeyUp {
                                key: Key::Space, ..
                            }) => true,
                            _ => false,
                        };

                        if started_once == false && !key_up {
                            grid_world.reset_pheromones();
                            started_once = true;
                            return;
                        }

                        // RESET AND RESTART THE WORLD
                        if !sandbox_mode && player_health <= 0 {
                            rummage_refresh_timer = 0;

                            current_wave = -1;
                            let mut to_despawn = Vec::new();
                            for (e, _) in world.query::<(&Ant,)>().iter() {
                                to_despawn.push(e);
                            }
                            for e in to_despawn {
                                let _ = world.despawn(e);
                            }

                            intro_interpolate = 0.0;
                            grid_world.initial_terrain_setup();
                            player_health = player_health_max;
                            frames_elapsed_since_last_wave = 0;
                            slider.clear(world);
                            wave_text_shown = false;
                            rummage_count = 0;
                            bullet_count = 0;
                            nacho_crumb_count = 0;
                            toothpaste_tile_count = 0;
                            straw_contents.clear();
                            straw_sucking = false;
                            defeated = false;
                            current_item = CurrentItem::Finger;
                            
                            inventory.clear();
                            inventory.insert(CurrentItem::Straw);

                            slider.hide(world);
                        } else {
                            let mut ui_state: std::sync::RwLockWriteGuard<'_, UIState> =
                                resources.get::<UIState>();

                            let option = slider.stop_line(world);

                            match option {
                                Some(o) => {
                                    match o {
                                        SliderOption::Bullets => {
                                            ui_state.bonus_text = "Found 6 BULLETS".into();
                                            bullet_count += 20;
                                            ui_state.display_bonus_text = 10.0;
                                        }
                                        SliderOption::Gun => {
                                            inventory.insert(CurrentItem::Gun);
                                            ui_state.bonus_text =
                                                "Found a GUN in my pocket\nPress 4 to equip".into();
                                            ui_state.display_bonus_text = 60.0;
                                        }
                                        SliderOption::Nachos => {
                                            nacho_crumb_count += 6;
                                            ui_state.bonus_text =
                                                "\"Chip crumbs. Perfect. \"\nPress 5 to equip"
                                                    .into();
                                            ui_state.display_bonus_text = 20.0;
                                        }
                                        SliderOption::Toothpaste => {
                                            inventory.insert(CurrentItem::Toothpaste);

                                            toothpaste_tile_count += 200;
                                            ui_state.bonus_text =
                                                "\"Toothpaste. Minty fresh barriers. \"\nPress 3 to equip"
                                                    .into();
                                            ui_state.display_bonus_text = 20.0;
                                        }
                                    }
                                    //slider.remove_random(world);
                                    rummage_count -= 1;
                                }

                                None => {}
                            }

                            if slider.visible {
                                ui_state.display_bonus_text = 10.0;
                                slider.hide(world);

                                /*
                                if rummage_count > 0 {
                                    ui_state.bonus_text =
                                        "I still feel an urge to rummage...".into();
                                    ui_state.display_bonus_text = 20.0;
                                }
                                */
                            } else if !key_up {
                                slider.show(world);

                                /*
                                if !slider.targets.is_empty()
                                /*&& rummage_count > 0*/
                                {
                                    slider.show(world);
                                } else {
                                    ui_state.bonus_text = "I'll rummage after more ants die".into();
                                    ui_state.display_bonus_text = 20.0;
                                }
                                */
                            }
                            println!("STOPPED ON: {:?}", option);
                        }
                    }
                    Event::KappEvent(KappEvent::KeyDown { key: Key::N, .. }) => {
                        spawn_wave_override = true;
                    }
                    Event::KappEvent(KappEvent::KeyDown {
                        key: Key::Digit1, ..
                    }) => {
                        current_item = CurrentItem::Finger;
                    }
                    Event::KappEvent(KappEvent::KeyDown { key: Key::I, .. }) => {
                        current_item = CurrentItem::Inspector;
                    }
                    Event::KappEvent(KappEvent::KeyDown {
                        key: Key::Digit3, ..
                    }) => {
                        let item = CurrentItem::Toothpaste;
                        if inventory.contains(&item) {
                            current_item = item;
                        } else {
                            let mut ui_state = resources.get::<UIState>();
                            ui_state.display_bonus_text = 50.0;
                            ui_state.bonus_text = "I'm out of that".into();
                        }
                    }
                    Event::KappEvent(KappEvent::KeyDown {
                        key: Key::Digit4, ..
                    }) => {
                        let item = CurrentItem::Gun;
                        if inventory.contains(&item) {
                            current_item = item;
                        }
                    }
                    Event::KappEvent(KappEvent::KeyDown {
                        key: Key::Digit5, ..
                    }) => {
                        let item = CurrentItem::Nacho;
                        if nacho_crumb_count > 0 {
                            current_item = item;
                        } else {
                            let mut ui_state = resources.get::<UIState>();
                            ui_state.display_bonus_text = 50.0;
                            ui_state.bonus_text = "I'm out of that".into();
                        }
                    }
                    Event::KappEvent(KappEvent::KeyDown {
                        key: Key::Digit2, ..
                    }) => {
                        let item = CurrentItem::Straw;
                        
                        if inventory.contains(&item) {
                            current_item = item;
                        } else {
                            let mut ui_state = resources.get::<UIState>();
                            ui_state.display_bonus_text = 50.0;
                            ui_state.bonus_text = "I'm out of that".into();
                        }
                    }
                    Event::KappEvent(KappEvent::KeyDown {
                        key: Key::Digit6, ..
                    }) => {
                        let item = CurrentItem::Spider;
                        if spider_count > 0 {
                            current_item = item;
                        } else {
                            let mut ui_state = resources.get::<UIState>();
                            ui_state.display_bonus_text = 50.0;
                            ui_state.bonus_text = "I'm out of that".into();
                        }
                    }
                    Event::FixedUpdate => {
                        if skip_first {
                            skip_first = false;
                            return;
                        }

                        let mut to_despawn: Vec<Entity> = Vec::new();

                        let mut audio_manager: std::sync::RwLockWriteGuard<'_, AudioManager> = resources.get::<AudioManager>();

                        slider.progress_line(world);

                        {
                            let mut ui_state = resources.get::<UIState>();
                            ui_state.display_bonus_text -= 0.1;
                            if ui_state.display_bonus_text < 0.0 {
                                ui_state.bonus_text = String::new();
                            }
                        }

                        ui_transparency_animation -= 0.005;

                      
                        if player_health > 0 && !sandbox_mode && !victory && started_once {
                            
                            rummage_refresh_timer -= 1;
                            if rummage_refresh_timer <= 0 && !slider.visible {
                                let random_amount_to_stash = random.range_u32(2..10);
                                // for _ in 0..random_amount_to_stash {
                                //     slider.remove_random(world);
                                // }

                                for _ in 0..(random_amount_to_stash.saturating_sub(4)) {
                                    slider.insert_random_lost_entry(world);
                                }

                                /*
                                let random_new_items = random.range_u32(0..4);
                                for _ in 0..random_new_items {
                                    let new_type = random.select_from_slice(&[SliderOption::Bullets, SliderOption::Nachos, SliderOption::Toothpaste]);
                                    
                                        match new_type {
                                            SliderOption::Bullets => {
                                                slider.add_option(
                                                    world,
                                                    0.02,
                                                    SliderOption::Bullets,
                                                    bullets_slider_material.clone(),
                                                );
                                            }
                                            SliderOption::Nachos => {
                                                slider.add_option(
                                                    world,
                                                    0.02,
                                                    SliderOption::Nachos,
                                                    nacho_slider_material.clone(),
                                                );
                                            }
                                            SliderOption::Toothpaste => {
                                                slider.add_option(
                                                    world,
                                                    0.15,
                                                    SliderOption::Toothpaste,
                                                    toothpaste_slider_color.clone(),
                                                );
                                            }
                                            _ => unreachable!()
                                        }   
                                }
                                */

                                if !slider.targets.is_empty() {
                                    let mut ui_state = resources.get::<UIState>();
                                   // ui_state.display_bonus_text = 50.0;
                                   // ui_state.bonus_text =
                                   //     "\"I should rummage again...\" [SPACEBAR]".into();
                                }
                                rummage_refresh_timer = rummage_refresh_reset;
                            }
                            

                            // Trigger waves
                            frames_elapsed_since_last_wave += 1;

                            wave_move_on_timer += 1;

                            let current_wave_done;
                            if current_wave == -1 {
                                current_wave_done = true;
                            } else {
                                if let Some(wave) = waves.get(current_wave as usize) {
                                    let trigger_met = match wave.victory_condition {
                                        WaveTrigger::LowAntCount(c) => {
                                            let mut count = 0;
                                            for (_, (_, t)) in
                                                world.query::<(&Ant, &Transform)>().iter()
                                            {
                                                if wave_move_on_timer > 60 * 30 {
                                                    if t.position.y < world_size.y - 10.0 {
                                                        count += 1;
                                                    }
                                                } else {
                                                    count += 1;
                                                }
                                            }
                                            c >= count
                                        }
                                        WaveTrigger::FramesElapsedSinceLast(c) => {
                                            c <= frames_elapsed_since_last_wave
                                        }
                                    };
                                    current_wave_done = trigger_met;
                                } else {
                                    current_wave_done = true;
                                }
                            }
                            

                            if current_wave_done || spawn_wave_override {
                                spawn_wave_override = false;
                                if let Some(wave) = waves.get((current_wave + 1) as usize) {
                                    if !wave_text_shown {
                                        ui_transparency_animation = 1.0;

                                        let mut ui_state = resources.get::<UIState>();
                                        ui_state.ui_lines = wave.ui_lines.clone();
                                    }

                                    if frames_elapsed_since_last_wave > wave.frame_delay as _ {
                                        wave_move_on_timer = 0;

                                        println!("SPAWNING WAVE: {:?}", current_wave);
                                        frames_elapsed_since_last_wave = 0;
                                        current_wave += 1;
                                        spawn_wave(world, wave);
                                        rummage_refresh_timer = 0;

                                        // Remove everything but it may come back.
                                        for _ in 0..slider.targets.len() {
                                            slider.remove_random(world);
                                        }

                                        for option in wave.rummage_options.iter() {
                                            match option {
                                                SliderOption::Gun => {
                                                    slider.add_option(
                                                        world,
                                                        0.3,
                                                        SliderOption::Gun,
                                                        gun_slider_material.clone(),
                                                    );
                                                }
                                                SliderOption::Bullets => {
                                                    slider.add_option(
                                                        world,
                                                        0.02,
                                                        SliderOption::Bullets,
                                                        bullets_slider_material.clone(),
                                                    );
                                                }
                                                SliderOption::Nachos => {
                                                    slider.add_option(
                                                        world,
                                                        0.02,
                                                        SliderOption::Nachos,
                                                        nacho_slider_material.clone(),
                                                    );
                                                }
                                                SliderOption::Toothpaste => {
                                                    slider.add_option(
                                                        world,
                                                        0.15,
                                                        SliderOption::Toothpaste,
                                                        toothpaste_slider_color.clone(),
                                                    );
                                                }
                                            }
                                        }

                                        if wave.bonus_text.len() > 0 {
                                            let mut ui_state = resources.get::<UIState>();
                                            ui_state.display_bonus_text = 50.0;
                                            ui_state.bonus_text = wave.bonus_text.clone();
                                        } else if slider.targets.len() > 0 {
                                           // let mut ui_state = resources.get::<UIState>();
                                           // ui_state.display_bonus_text = 50.0;
                                           // ui_state.bonus_text =
                                           //     "[Hold SPACE to rummage in pockets]".into();
                                        }

                                        let random_amount = random.range_u32(1..10);
                                        for _ in 0..random_amount {
                                            slider.insert_random_lost_entry(world);
                                        }

                                        rummage_count = 2;

                                        wave_text_shown = false;

                                        ui_transparency_animation = 1.0;
                                    }
                                } else {
                                    // No more waves!
                                    victory = true;
                                }
                            }
                        }

                        gun_cooldown_animation -= 0.1;

                        // Intro Sequence Stuff

                        if started_once && !sandbox_mode {
                            intro_interpolate += 0.002;

                            if intro_interpolate < 1.0 {
                                let mut camera: RefMut<'_, Transform> =
                                    world.get::<&mut Transform>(camera).unwrap();
                                let curve = koi3::animation_curves::smooth_step(intro_interpolate);
                                camera.position.y = camera_max + (camera_min - camera_max) * curve;
                            }
                            resources.get::<UIState>().transparency =
                                intro_interpolate.min(1.0 - ui_transparency_animation);
                        }

                        let input: &mut std::sync::RwLockWriteGuard<'_, kapp::StateTracker> =
                            &mut resources.get::<Input>();
                        let pointer_position_window: (f64, f64) = input.pointer_position();
                        let pointer_position_new = get_pointer_world_position(
                            world,
                            &resources,
                            camera_child,
                            pointer_position_window.0 as _,
                            pointer_position_window.1 as _,
                        );

                        let max_pointer_speed = pointer_max_speed.unwrap_or(100.0);

                        // Adjust pointer speed
                        {
                            //  if let Some(pointer_max_speed) = pointer_max_speed {
                            let dir = pointer_position_new - pointer_position;
                            let length = dir.length();

                            let max_scale = (length / 200.0).clamp(0.0, 1.0);

                            let amount = max_scale * max_pointer_speed;

                            if length != 0.0 {
                                let dir = dir / length;
                                pointer_position += dir * amount;
                            }
                        }
                        // } else {
                        //     pointer_position = pointer_position_new;
                        // }

                        match current_item {
                            CurrentItem::Gun | CurrentItem::Straw => {
                                world
                                    .get::<&mut Transform>(reticule_entity)
                                    .unwrap()
                                    .position = pointer_position_new.xy().extend(-10.0);

                                *world.get::<&mut RenderFlags>(reticule_entity).unwrap() =
                                    RenderFlags::DEFAULT;
                            }
                            _ => {
                                *world.get::<&mut RenderFlags>(reticule_entity).unwrap() =
                                    RenderFlags::NONE;
                            }
                        }

                        let max_hand_reach = 130.0;
                        if player_health > 0 || sandbox_mode {
                            match current_item {
                                CurrentItem::Spider => {
                                    if !sandbox_mode {
                                        pointer_position.y = pointer_position.y.min(max_hand_reach);
                                    }

                                    *world.get::<&mut Handle<Material>>(player_item_art).unwrap() =
                                        spiderhand.clone();
                                
                                    if input.pointer_button_down(PointerButton::Primary) {
                                        if spider_count > 0 {
                                            spider_count -= 1;
                                            world.spawn((
                                                Transform::new().with_position(pointer_position).with_scale(Vec3::fill(2.0)),
                                                Ant {
                                                    dir: -Vec2::Y,
                                                    speed: random.range_f32(0.05..0.2),
                                                    return_mode: false,
                                                    health: 255,
                                                    carrying: None,
                                                    is_spider: true,
                                                    home: pointer_position.xy()
                                                },
                                                Mesh::VERTICAL_QUAD,
                                                //Material::UNLIT,
                                                spider_material.clone(),
                                            ));
                                        }
                                    }
                                }
                                CurrentItem::Straw => {
                                    *world.get::<&mut Handle<Material>>(player_item_art).unwrap() =
                                        strawhand.clone();
                                    
                                    let mut origin = pointer_position;
                                    if !sandbox_mode {
                                        origin.y = pointer_position.y.min(max_hand_reach);
                                    }
                                    let radius = 3.0;
                                    let height_increments = 30;

                                    let tiles_in_spitball = 16;
                                    if input.pointer_button_released(PointerButton::Primary) && pointer_held_len < 20 {
                                        println!("CAPACITY: {:?}", straw_contents.len());
                                        if !straw_sucking && !straw_contents.is_empty() {
                                            // Shoot back material
                                            
                                            let mut material = Vec::new();

                                            for _ in 0..tiles_in_spitball {
                                                if let Some(mat) = straw_contents.pop_back() {
                                                    material.push(mat);
                                                }
                                            }

                                            world.spawn((
                                                Transform::new().with_position(origin.xy().extend(-8.0)).with_scale(Vec3::fill(2.0)),
                                                Mesh::VERTICAL_CIRCLE,
                                                Material::UNLIT,
                                                SpitBall {
                                                    target: pointer_position_new.xy(),
                                                    material
                                                }
                                            ));

                                            {
                                                let sounds = resources.get::<AssetStore<Sound>>();
                                                let sound = sounds.get(random.select_from_slice(&spit_sounds));
                                                audio_manager.play_one_shot_with_speed(sound, random.range_f32(0.8..1.2));
                                            }
                                        }
                                    }

                                    let straw_capacity = tiles_in_spitball * 15;

                                    if input.pointer_button(PointerButton::Primary) && pointer_held_len > 20 {
                                        if straw_contents.len() < straw_capacity || straw_sucking {
                                            if current_straw_sound.is_none() {
                                                let sounds = resources.get::<AssetStore<Sound>>();
                                                let sound = sounds.get(&suck_sound);
                                                
                                                current_straw_sound = Some(audio_manager
                                                    .play_one_shot_oddio(oddio::MonoToStereo::new(
                                                        oddio::Cycle::new(sound.frames.clone())
                                                )));
                                            }

                                            if straw_contents.len() < straw_capacity {
                                                for (e, (transform, ant)) in
                                                    world.query::<(&mut Transform, &mut Ant)>().iter() {
                                                        let dir = transform.position - origin;
                                                        let distance = dir.length();
                                                        let normalized_dir = dir / distance;

                                                        if distance < 2.0 {
                                                            to_despawn.push(e);

                                                            player_hurt_effect += 1.0;
                                                            screen_shake_amount += 2.0;
                                                            player_health -= 1;

                                                            let sounds = resources.get::<AssetStore<Sound>>();
                                                            let sound = sounds.get(random.select_from_slice(&squish_sounds));
                                                            audio_manager.play_one_shot_with_speed(sound, random.range_f32(0.3..0.5));
                            

                                                            for _ in 0..10 {
                                                                straw_contents.push_back(TileMaterial::AntBody);
                                                            }
                                                        }

                                                        let suck_radius = 10.0;
                                                        if distance < suck_radius {
                                                            transform.position -= normalized_dir * 0.9 * (1.0 - distance / suck_radius);
                                                        }
                                                }
                                            }
                                            // TODO: Check for ants
                                            straw_sucking = true;
                                            grid_world.for_tile_in_radius(
                                                origin.xy(),
                                                radius,
                                                |i, _, t| {

                                                    // Heal from chip crumbs
                                                    if t.material == TileMaterial::Nacho {
                                                        player_hurt_effect -= 1.0;
                                                        screen_shake_amount += 0.1;
                                                        if player_health < 50 {
                                                            player_health += 4;
                                                        }
                                                        t.material = TileMaterial::Earth;
                                                    }

                                                    if straw_contents.len() < straw_capacity && random.f32() > 0.96 {
                                                        if t.height > height_increments as u8 {
                                                            t.height = t.height.saturating_sub(height_increments * 1.4 as u8);
                                                            straw_contents.push_back(t.material);
                                                        }

                                                    }
                                                    false
                                                },
                                            );
                                        }
                                    } else {
                                        straw_sucking = false;
                                    }

                                    if ! straw_sucking || straw_contents.len() >= straw_capacity {
                                        if let Some(mut current_straw_sound) = current_straw_sound.take() {
                                            current_straw_sound.control::<oddio::Stop<_>, _>().stop();
                                        }
                                    }

                                    pointer_position = origin;
                                    
                                }
                                CurrentItem::Inspector => {
                                    if input.pointer_button_down(PointerButton::Primary) {
                                        let radius = 1.0;

                                        grid_world.for_tile_in_radius(
                                            pointer_position_new.xy(),
                                            radius,
                                            |i, _, t| {
                                                println!("P: {:?}", t.pheremone_level);
                                                println!("H: {:?}", t.height);

                                                false
                                            },
                                        );
                                    }
                                }
                                CurrentItem::Nacho => {
                                    if !sandbox_mode {
                                        pointer_position.y = pointer_position.y.min(max_hand_reach);
                                    }

                                    *world.get::<&mut Handle<Material>>(player_item_art).unwrap() =
                                        nacho_hand.clone();

                                    if input.pointer_button_down(PointerButton::Primary) {
                                        let radius = 2.0;

                                        let mut tile_to_drop_pheromone = Vec::new();
                                        grid_world.for_tile_in_radius(
                                            pointer_position_new.xy(),
                                            radius,
                                            |i, _, t| {
                                                if nacho_crumb_count > 0
                                                    && t.material != TileMaterial::Nacho
                                                    && random.f32() > 0.90
                                                {
                                                    t.material = TileMaterial::Nacho;
                                                    nacho_crumb_count -= 1;
                                                    tile_to_drop_pheromone.push(i)
                                                }

                                                false
                                            },
                                        );

                                        let nacho_pheromone_radius = 20.0;
                                        for t in tile_to_drop_pheromone {
                                            grid_world.for_tile_in_radius(
                                                pointer_position_new.xy(),
                                                nacho_pheromone_radius,
                                                |i, r, t| {
                                                    t.pheremone_level -=
                                                        (nacho_pheromone_radius - r) as i16;

                                                    false
                                                },
                                            );
                                        }
                                    }
                                    if nacho_crumb_count <= 0 {
                                        inventory.remove(&CurrentItem::Nacho);
                                        current_item = CurrentItem::Finger;
                                    }
                                }
                                CurrentItem::Gun => {
                                    if gun_cooldown_animation <= 0.0 {
                                        *world
                                            .get::<&mut Handle<Material>>(player_item_art)
                                            .unwrap() = pointed_gun.clone();
                                    } else {
                                        *world
                                            .get::<&mut Handle<Material>>(player_item_art)
                                            .unwrap() = gun_fire.clone();
                                    }

                                    if input.pointer_button_down(PointerButton::Primary) {
                                        
                                        if bullet_count > 0 {
                                            bullet_count -= 1;
                                            let blast_radius = 6.0;
                                            grid_world.for_tile_in_radius(
                                                pointer_position_new.xy(),
                                                blast_radius,
                                                |_, r, t| {
                                                    let how_close_to_center =
                                                        1.0 - (r / blast_radius);
                                                    let scale = random.f32() * how_close_to_center;

                                                    if scale > 0.1 {
                                                        t.height = t
                                                            .height
                                                            .saturating_sub((150.0 * scale) as _);
                                                        t.material = TileMaterial::Earth;
                                                    }

                                                    t.kill_flag = true;
                                                    true
                                                },
                                            );


                                            {
                                                let sounds = resources.get::<AssetStore<Sound>>();
                                                let sound = sounds.get(random.select_from_slice(&gun_sounds));
                                                audio_manager.play_one_shot_with_speed(sound, random.range_f32(0.8..1.2));
                                            }

                                            screen_shake_amount += 3.0;
                                            gun_cooldown_animation = gun_cooldown_animation_reset;

                                            let distance_to_left_foot = (pointer_position.xy()
                                                - Vec2::new(40.0, 37.0))
                                            .abs();
                                            let distance_to_right_foot = (pointer_position.xy()
                                                - Vec2::new(60.5, 36.0))
                                            .abs();

                                            if (distance_to_left_foot.x < 5.0
                                                && distance_to_left_foot.y < 13.0)
                                                || (distance_to_right_foot.x < 5.0
                                                    && distance_to_right_foot.y < 13.0)
                                            {
                                                {
                                                    let sounds = resources.get::<AssetStore<Sound>>();
                                                    let sound = sounds.get(random.select_from_slice(&squish_sounds));
                                                    audio_manager.play_one_shot_with_speed(sound, random.range_f32(0.3..0.4));                
                                                }
                                                println!("P: {:?}", pointer_position.xy());
                                                // SHOT FOOT
                                                player_hurt_effect += 4.0;
                                                screen_shake_amount += 40.0;
                                                player_health -= 10;
                                            }
                                        } else {
                                            // TODO: Play click sound.
                                            {
                                                let sounds = resources.get::<AssetStore<Sound>>();
                                                let sound = sounds.get(random.select_from_slice(&out_of_bullets_sounds));
                                                audio_manager.play_one_shot_with_speed(sound, random.range_f32(0.9..1.1));
                                            }
                                        }
                                    }
                                    pointer_max_speed = None;

                                    if !sandbox_mode {
                                        pointer_position.y = pointer_position.y.min(max_hand_reach);
                                    }
                                }
                                CurrentItem::Toothpaste => {
                                    *world
                                        .get::<&mut Handle<Material>>(player_item_art)
                                        .unwrap() = toothpaste_hand.clone();
                                    if !sandbox_mode {
                                        pointer_position.y = pointer_position.y.min(max_hand_reach);
                                    }

                                    let max_squeeze = 1.0;

                                    if input.pointer_button_down(PointerButton::Primary) {
                                        toothpaste_squeeze = max_squeeze;
                                    }

                                    if input.pointer_button(PointerButton::Primary) {
                                        let radius = 2.0 * (toothpaste_squeeze / max_squeeze);
                                        grid_world.for_tile_in_radius(
                                            pointer_position.xy(),
                                            radius,
                                            |_, r, t| {
                                                match t.material {
                                                    TileMaterial::ToothPaste1 | TileMaterial::ToothPaste2 | TileMaterial::ToothPaste3 => {
                                                        return false;
                                                    }
                                                    _ => {}
                                                }

                                                if toothpaste_tile_count <= 0 {
                                                    toothpaste_tile_count = 0;
                                                    return false;
                                                }
                                                toothpaste_tile_count -= 1;

                                                t.height = 255;
                                                t.height = (((1.0 - (r / radius)) * 200.0) as u8)
                                                    .max(t.height);

                                                match random.range_u32(0..3) {
                                                    0 => {
                                                        t.material = TileMaterial::ToothPaste1;
                                                    }
                                                    1 => {
                                                        t.material = TileMaterial::ToothPaste2;
                                                    }
                                                    2 => {
                                                        t.material = TileMaterial::ToothPaste3;
                                                    }
                                                    _ => t.material = TileMaterial::ToothPaste1,
                                                }
                                                true
                                            },
                                        );

                                        //  toothpaste_squeeze -= 0.02;
                                        toothpaste_squeeze = toothpaste_squeeze.max(0.0);
                                        pointer_max_speed = Some(10.0);
                                    } else {
                                        pointer_max_speed = None;
                                    }
                                }
                                CurrentItem::Finger => {
                                    // Max distance a hand can reach
                                    if !sandbox_mode {
                                        pointer_position.y = pointer_position.y.min(max_hand_reach);
                                    }
                                    if input.pointer_button(PointerButton::Primary) {
                                        *world
                                            .get::<&mut Handle<Material>>(player_item_art)
                                            .unwrap() = pointed_finger_material.clone();

                                        let slide_scale =
                                            (pointer_position - pointer_last_position).length();

                                        //println!("XY: {:?}", pointer_position.xy());

                                        grid_world.for_tile_in_radius(
                                            pointer_position.xy(),
                                            3.0,
                                            |_, _, t| {
                                                if slide_scale > 0.2 {
                                                    let amount = 14.0 * (slide_scale / 3.0);
                                                    t.height = t.height.saturating_sub(amount as _);
                                                }

                                                if t.height < 30 {
                                                    t.material = TileMaterial::Earth;
                                                }

                                                // t.material = TileMaterial::AntBody;

                                                if input.pointer_button_down(PointerButton::Primary)
                                                {
                                                    t.kill_flag = true;
                                                }
                                                true
                                            },
                                        );
                                    } else {
                                        *world
                                            .get::<&mut Handle<Material>>(player_item_art)
                                            .unwrap() = curled_finger_material.clone();
                                    }
                                }
                            }
                        }

                        // Update user input.
                        world.get::<&mut Transform>(player_item).unwrap().position =
                            pointer_position;


                        // Move spitballs
                        for (e, (transform, spitball)) in
                            world.query::<(&mut Transform, &mut SpitBall)>().iter()
                        {
                            let dir = (spitball.target - transform.position.xy()).normalized();

                            let spitball_speed = 0.8;
                            transform.position += (dir * spitball_speed).extend(0.0);
                            if (transform.position.xy() - spitball.target).length() < 1.0 {
                                to_despawn.push(e);
                                println!("REACHED DESTINATION!");
                                grid_world.for_tile_in_radius(transform.position.xy(), 3.0, |tile_p, _, tile| {
                                    if let Some(material) = spitball.material.pop() {
                                        tile.height += 100;
                                        tile.material = material;
                                    }
                                    tile.kill_flag = true;
                                    true
                                });
                            }
                        }

                        // Ant Behavior
                        for (e, (transform, ant)) in
                            world.query::<(&mut Transform, &mut Ant)>().iter()
                        {
                            let p = transform.position;
                            let mut height_here = 0;

                            let mut will_die = false;

                            if let Some(tile_here) = grid_world.get_tile(p.x, p.y) {
                                if !ant.is_spider {
                                    if ant.return_mode {
                                        tile_here.pheremone_level -= 1;
                                        tile_here.height -= 1;
                                        tile_here.times_changed += 1;
                                    } else {
                                        tile_here.pheremone_level += 1;
                                        tile_here.times_changed += 1;
                                    }
                                    if tile_here.height > 150 && random.f32() > 0.5 {
                                        tile_here.height = tile_here.height.saturating_sub(1);
                                    }
                                    height_here = tile_here.height;

                                    // Consider dropping material
                                    if ant.return_mode
                                        && p.y > world_size.y - 20.0
                                        && random.f32() > 0.95
                                    {
                                        if tile_here.material == TileMaterial::Earth {
                                            ant.return_mode = false;

                                            if let Some(carrying) = ant.carrying.take() {
                                                tile_here.material = carrying;
                                                //tile_here.height += 10;
                                            }
                                        }
                                    }
                                

                                    // Consider picking up things.
                                    if p.y < 180.0 && ant.carrying.is_none() {
                                        match tile_here.material {
                                            TileMaterial::AntBody | TileMaterial::Nacho => {
                                                ant.return_mode = true;
                                                ant.carrying = Some(tile_here.material);
                                                tile_here.material = TileMaterial::Earth;
                                            }
                                            _ => {}
                                        }
                                    }

                                    // Toothpaste hurts!
                                    match tile_here.material {
                                        TileMaterial::ToothPaste1
                                        | TileMaterial::ToothPaste2
                                        | TileMaterial::ToothPaste3 => {
                                            // ant.health = ant.health.saturating_sub(2);
                                        }
                                        _ => {}
                                    }
                                }


                                if tile_here.kill_flag || (!ant.is_spider && tile_here.kill_ants_only) {
                                    will_die = true;

                                    // Killed violently, scatter guts.
                                    grid_world.for_tile_in_radius(p.xy(), 2.0, |_tile_p, _, t| {
                                        if random.f32() > 0.8 {
                                            t.material = TileMaterial::AntGuts;
                                            t.height += 5;
                                        }
                                        false
                                    });

                                    let sounds = resources.get::<AssetStore<Sound>>();
                                    let sound = sounds.get(random.select_from_slice(&squish_sounds));
                                    audio_manager.play_one_shot_with_speed(sound, random.range_f32(1.0..2.0));
                                    
                                }


                                if ant.is_spider {
                                    grid_world.for_tile_in_radius(p.xy(), 10.0, |_tile_p, _, t| {
                                        t.kill_ants_only = true;
                                        true
                                    });
                                }
                            }

                            // The ant moves towards the tile with the lowest score.
                            let mut lowest_score = i32::MAX;

                            let mut lowest_score_tile = None;
                            let mut height_speed_multiplier = 1.0;

                            grid_world.for_tile_in_radius(p.xy(), 4.0, |tile_p, _, t| {
                                let alignment = (p.xy() - tile_p).dot(ant.dir);
                                let alignment_score_adjust = if alignment > 0.0 { 300 } else { 0 };

                                let height_diff = t.height as f32 - height_here as f32;

                                let height_score_adjust = if height_diff < 50.0 {
                                    // Next tile is lower
                                    (height_diff.abs() / 255.0) * 100.0
                                } else {
                                    // Next tile is greater
                                    (height_diff.abs() / 255.0) * 1000.0
                                };

                                let pheremone_score = if ant.return_mode {
                                    -(t.pheremone_level as i32)
                                } else {
                                    t.pheremone_level as i32
                                };

                                let mut tile_score = pheremone_score
                                    + height_score_adjust as i32
                                    + alignment_score_adjust;

                                // Random score adjust
                                // I believe this biases towards right tiles.
                                // tile_score += random.range_i32(0..4);

                                if tile_p.y > world_size.y - 5.0 {
                                    tile_score = i32::MAX;
                                }

                                if ant.is_spider {
                                    if (tile_p.xy() - ant.home).length() > 50.0 {
                                        tile_score = (((tile_p.xy() - ant.home).length() / 30.0) * 100.0) as i32;
                                    } else {
                                        tile_score = -(t.times_changed as i32);
                                    }

                                }

                                let height_veto = t.height > 230 && random.f32() > 0.01;

                                if !height_veto && tile_score < lowest_score {
                                    lowest_score = tile_score;
                                    lowest_score_tile = Some(tile_p);

                                    height_speed_multiplier = (1.0
                                        - (height_score_adjust / 20.0).min(1.0))
                                    .clamp(0.5, 1.0);
                                }
                                false
                            });

                         

                            // println!("LOWEST SCORE TILE: {:?}", lowest_score_tile);
                            // println!("ANT P: {:?}", p.xy());
                            // println!("SCORE: {:?}", lowest_score);

                            if let Some(lowest_score_tile) = lowest_score_tile {
                                let tile_p = p.xy().as_i32().as_f32();

                                let unnormalized_dir = lowest_score_tile - tile_p.xy();
                                let len = unnormalized_dir.length();

                                if len != 0.0 {
                                    let new_dir = unnormalized_dir / len;
                                    //ant.target_score = lowest_pheremone_v;
                                    // println!("P: {:?}", p);
                                    ant.dir = new_dir;
                                    // ant.dir = -Vec2::Y;
                                    // println!("FORWARD: {:?}", transform.forward());
                                    let angle = ant.dir.y.atan2(ant.dir.x);
                                    transform.rotation = Quaternion::from_angle_axis(
                                        angle - std::f32::consts::PI / 2.0,
                                        Vec3::Z,
                                    );
                                } else {
                                    ant.dir = Vec2::ZERO;
                                }
                            } else {
                                ant.dir = Vec2::ZERO;
                            }

                            // can't work due to borrowing rules
                            /*
                            if ant.is_spider {
                                let mut closest = None;
                                let mut closest_d = f32::MAX;
                                for (e, (a, t)) in world.query::<(&Ant,&Transform)>().iter() {
                                    if !a.is_spider {
                                        let length_squared = (t.position.xy() - p.xy()).length_squared();
                                        if length_squared < 50.0 * 50.0 && length_squared < closest_d {
                                            closest_d = length_squared;
                                            closest = Some(t.position.xy());
                                        }
                                    }
                                }

                                if let Some(closest) = closest {
                                    ant.dir = (closest - p.xy()).normalized();
                                }
                            }
                            */

                            //   println!("FINAL DIR: {:?}", ant.dir);

                            let mut speed_multiplier = 1.0;
                            if ant.return_mode {
                                speed_multiplier *= 1.5
                            }


                            if transform.position.y > world_size.y - 1.0 {
                                ant.dir = -Vec2::Y;
                            }

                            if ant.health < 100 {
                                if random.f32() > 0.8 {
                                    ant.health = ant.health.saturating_sub(1)
                                }
                            }

                            if ant.health == 0 {
                                will_die = true;
                            }

                            if !sandbox_mode && !victory
                                && player_health > 0
                                && transform.position.y > 15.0
                                && (transform.position.xy() - Vec2::new(world_size.x / 2.0, 15.0))
                                    .length()
                                    < 10.0
                            {
                                // Hurt player
                                player_health -= 1;
                                screen_shake_amount += 1.0;
                                player_hurt_effect += 1.0;

                                let sounds = resources.get::<AssetStore<Sound>>();
                                let sound = sounds.get(random.select_from_slice(&squish_sounds));
                                audio_manager.play_one_shot_with_speed(sound, random.range_f32(0.3..0.5));

                                println!("ANT GOT PASSED: {:?}", player_health);

                                to_despawn.push(e);
                            }

                            if will_die {
                                to_despawn.push(e);

                                grid_world.for_tile_in_radius(p.xy(), 1.0, |_tile_p, _, t| {
                                    if random.f32() > 0.4 {
                                        t.material = TileMaterial::AntBody;
                                        t.height = 155;
                                    }
                                    false
                                });

                                // Ant death pheromones to attract more ants.
                                let ant_pheremone_radius = 10.0;
                                grid_world.for_tile_in_radius(
                                    p.xy(),
                                    ant_pheremone_radius,
                                    |_tile_p, radius, t| {
                                        t.pheremone_level -=
                                            (5.0 * (1.0 - (radius / ant_pheremone_radius))) as i16;
                                        t.pheremone_level = t.pheremone_level.max(0);
                                        false
                                    },
                                );

                                // Drop what it's carrying if it dies.
                                let tile_here = grid_world.get_tile(p.x, p.y).unwrap();
                                if let Some(carrying) = ant.carrying.take() {
                                    tile_here.material = carrying;
                                }
                            }

                            let health_speed_multiplier = ant.health as f32 / 255.0;


                            if ant.is_spider {
                                speed_multiplier = 5.0;;
                            }

                            transform.position += ant.dir.extend(0.0)
                                * ant.speed
                                * speed_multiplier
                                * height_speed_multiplier
                                * health_speed_multiplier;
                        }

                    

                        // println!("P: {:?}", pointer_position);

                        // Scroll the camera if needed.

                        if !victory && player_health > 0 && intro_interpolate > 1.0 {
                            let window = resources.get::<kapp::Window>();
                            let (_window_width, window_height) = window.size();
                            let window_position =
                                pointer_position_window.1 as f32 / window_height as f32;

                            let mut camera: RefMut<'_, Transform> =
                                world.get::<&mut Transform>(camera).unwrap();

                            let camera_move_rate = 1.0;
                            let camera_buffer_area = 0.1;
                            if window_position < camera_buffer_area {
                                let multiplier =
                                    1.0 - (window_position / camera_buffer_area).min(1.0);
                                camera.position.y += camera_move_rate * multiplier * multiplier;
                            } else if window_position > (1.0 - camera_buffer_area) {
                                let multiplier = ((window_position - (1.0 - camera_buffer_area))
                                    / camera_buffer_area)
                                    .min(1.0);

                                camera.position.y -= camera_move_rate * multiplier * multiplier;
                            }

                            //println!("CAMERA POSITION Y: {:?}", camera.position.y);
                            camera.position.y = camera.position.y.clamp(camera_min, 160.0);
                        }

                        grid_world.update_tiles();
                        pointer_last_position = pointer_position;

                        for e in to_despawn {
                            let _ = world.despawn(e);
                        }

                        if input.pointer_button(PointerButton::Primary) {
                            pointer_held_len += 1;
                        } else {
                            pointer_held_len = 0;
                        }
                    }
                    _ => {}
                }
            }
        });
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CurrentItem {
    Finger,
    Straw,
    Nacho,
    Toothpaste,
    Gun,
    Spider,
    Inspector,
}

struct GridWorld {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    texture_scratch: Vec<[f32; 4]>,
    tiles_to_update: VecDeque<usize>,
}
#[derive(Clone, Copy)]
struct Tile {
    height: u8,
    material: TileMaterial,
    /// How much ants have explored here.
    pheremone_level: i16,
    kill_flag: bool,
    kill_ants_only: bool,
    times_changed: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]

enum TileMaterial {
    Earth,
    Grass,
    AntGuts,
    AntBody,
    ToothPaste1,
    ToothPaste2,
    ToothPaste3,
    Nacho,
    Debug,
}

impl GridWorld {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![
                Tile {
                    height: 128,
                    material: TileMaterial::Earth,
                    pheremone_level: 0,
                    kill_flag: false,
                    kill_ants_only: false,
                    times_changed: 0
                };
                width * height
            ],
            texture_scratch: Vec::new(),
            tiles_to_update: VecDeque::new(),
        }
    }

    pub fn update_tiles(&mut self) {
        while let Some(i) = self.tiles_to_update.pop_back() {
            let tile = &mut self.tiles[i];
            tile.kill_flag = false;
            tile.kill_ants_only = false;
        }
    }

    pub fn reset_pheromones(&mut self) {
        let mut r = Random::new();
        for (i, t) in self.tiles.iter_mut().enumerate() {
            let x = i / self.height;
            let y = i % self.height;

            let base_level = 1000;
            let height_pheremone_offset = (y as f32 / self.height as f32) * 200.0;
            t.pheremone_level =
                height_pheremone_offset as i16 + r.range_u32(0..20) as i16 + base_level;

            let distance_to_feet =
                (Vec2::new(x as f32, y as f32) - Vec2::new(self.width as f32 / 2.0, 13.0)).length();

            t.pheremone_level += distance_to_feet as i16 * 2;

            if y > self.height - 3 {
                t.pheremone_level = i16::MAX;
            }
            t.times_changed = 0;
        }
    }

    pub fn neutral_pheremones(&mut self) {
        for (_, t) in self.tiles.iter_mut().enumerate() {
            t.pheremone_level = 0;
            t.times_changed = 0;
        }
    }

    pub fn reverse_deterent_pheromones(&mut self) {
        let mut r = Random::new();
        for (i, t) in self.tiles.iter_mut().enumerate() {
            let y = i % self.height;

            if y < self.height - 60 {
                t.pheremone_level = i16::MAX;
            }
            t.times_changed = 0;
        }
    }

    pub fn initial_terrain_setup(&mut self) {
        let noise2d: clatter::Simplex2d = clatter::Simplex2d::new();

        for (i, t) in self.tiles.iter_mut().enumerate() {
            t.material = TileMaterial::Earth;
            let x = i / self.height;
            let y = i % self.height;

            let scale = 30.0;
            let sample2d =
                (sample_with_octaves::<8>(&noise2d, 0.5, x as f32 / scale, y as f32 / scale) + 1.0)
                    / 2.0;

            let terrain_type = (sample_with_octaves::<4>(
                &noise2d,
                0.5,
                x as f32 / scale + 3000.0,
                y as f32 / scale + 7000.0,
            ) + 1.0)
                / 2.0;

            if terrain_type > 0.8 {
                t.material = TileMaterial::Grass;
            }

            let offset = 150.0;
            let height = (sample2d * offset) + (255.0 - offset);
            t.height = height as u8;
            t.times_changed = 0;
        }

        self.reset_pheromones();
    }

    pub fn get_tile(&mut self, x: f32, y: f32) -> Option<&mut Tile> {
        if x < 0.0 || y < 0.0 || x > self.width as _ || y > self.height as _ {
            return None;
        }
        let i = x as usize * self.height + y as usize;
        self.tiles.get_mut(i)
    }

    pub fn for_tile_in_radius(
        &mut self,
        center: Vec2,
        radius: f32,
        mut f: impl FnMut(Vec2, f32, &mut Tile) -> bool,
    ) {
        let min: Matrix<f32, 2, 1> = center - Vec2::fill(radius);
        let max: Matrix<f32, 2, 1> = center + Vec2::fill(radius);

        let min = min.as_usize();
        let max = max
            .as_usize()
            .max(Vector::<_, 2>::new(self.width, self.height));

        for x in min.x..max.x {
            for y in min.y..max.y {
                let t = Vec2::new(x as f32, y as f32);
                let i = x * self.height + y;
                let distance = (t - center).length();
                if distance < radius {
                    if let Some(tile) = self.tiles.get_mut(i) {
                        if f(Vec2::new(x as f32, y as f32), distance, tile) {
                            self.tiles_to_update.push_back(i);
                        }
                    }
                }
            }
        }
    }

    pub fn update_texture(&mut self, world: &mut World, resources: &Resources, entity: Entity) {
        let graphics = &mut resources.get::<Renderer>().raw_graphics_context;

        if true
        /*self.texture_scratch.len() >= 0*/
        {
            self.texture_scratch.clear();
            self.texture_scratch.reserve(self.width * self.height);
            for y in 0..self.height {
                for x in 0..self.width {
                    let t = self.tiles[x * self.height + (self.height - 1 - y)];
                    // println!("V: {:?}", x * self.height + y);
                    let c = match t.material {
                        TileMaterial::Earth => Color::from_srgb_hex(0x7A613B, 1.0),
                        TileMaterial::AntGuts => Color::YELLOW,
                        TileMaterial::AntBody => Color::GREEN,
                        TileMaterial::Debug => Color::MAGENTA,
                        TileMaterial::Nacho => Color::ORANGE,
                        TileMaterial::Grass => Color::from_srgb_hex(0xAAB55B, 1.0),
                        TileMaterial::ToothPaste1 => Color::from_srgb_hex(0x2589D0, 1.0),
                        TileMaterial::ToothPaste2 => Color::from_srgb_hex(0x2589D0, 1.0),
                        TileMaterial::ToothPaste3 => Color::from_srgb_hex(0xE98EC5, 1.0),
                    };

                    let c = Color::interpolate(
                        Color::from_srgb_hex(0x2A1605, 1.0),
                        c,
                        (t.height as f32 / 255.0).clamp(0.4, 1.0),
                    )
                    .to_srgb();

                    self.texture_scratch.push([c[0], c[1], c[2], c[3]]);

                    // data.push([
                    //     x as f32 / self.width as f32,
                    //     0.0, //y as f32 / self.height as f32,
                    //     0.0,
                    //     1.0,
                    // ]);
                }
            }
        }

        /*
        let data: Vec<_> = self
            .tiles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let c = match t.material {
                    TileMaterial::Earth => Color::interpolate(
                        Color::from_srgb_hex(0x2A1605, 1.0),
                        Color::from_srgb_hex(0x7A613B, 1.0),
                        (t.height as f32 / 200.0).clamp(0.4, 1.0),
                    )
                    .to_srgb(),
                };

                [c[0], c[1], c[2], c[3]]
            })
            .collect();
        */

        let new_texture = graphics.new_texture_with_data(
            self.width as _,
            self.height as _,
            1,
            &self.texture_scratch,
            TextureSettings {
                minification_filter: FilterMode::Nearest,
                magnification_filter: FilterMode::Nearest,
                generate_mipmaps: false,
                ..Default::default()
            },
        );

        let texture = resources
            .get::<AssetStore<Texture>>()
            .add(Texture(new_texture));

        let material = resources.get::<AssetStore<Material>>().add(Material {
            base_color_texture: Some(texture),
            shader: Shader::UNLIT,
            ..Default::default()
        });

        let _ = world.insert_one(entity, material);
    }
}

fn sample_with_octaves<const LANES: usize>(
    noise: &clatter::Simplex2d,
    persistence: f32,
    x: f32,
    y: f32,
) -> f32
where
    std::simd::LaneCount<LANES>: std::simd::SupportedLaneCount,
{
    use std::simd::num::SimdFloat;

    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    let mut amplitudes: [f32; LANES] = [0.0; LANES];
    let mut frequencies: [f32; LANES] = [0.0; LANES];

    for i in 0..LANES {
        amplitudes[i] = amplitude;
        frequencies[i] = frequency;

        max_value += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    let amplitudes = core::simd::Simd::<f32, LANES>::from_array(amplitudes);
    let frequencies = core::simd::Simd::<f32, LANES>::from_array(frequencies);
    let sample = noise.sample([
        core::simd::Simd::<f32, LANES>::splat(x) * frequencies,
        core::simd::Simd::<f32, LANES>::splat(y) * frequencies,
    ]) * amplitudes;

    sample.value.reduce_sum() / max_value
}

fn get_pointer_world_position(
    world: &World,
    resources: &Resources,
    camera_entity: Entity,
    x: f32,
    y: f32,
) -> Vec3 {
    use std::ops::Deref;

    let window = resources.get::<kapp::Window>();
    let (window_width, window_height) = window.size();

    let entity_ref = world.entity(camera_entity).unwrap();
    let camera = entity_ref.get::<&Camera>().unwrap();
    let camera_transform = entity_ref.get::<&GlobalTransform>().unwrap();

    let mut p = camera
        .view_to_ray(
            camera_transform.deref(),
            x,
            y,
            window_width as f32,
            window_height as f32,
        )
        .origin;

    p.z = 0.0;
    p
}

fn get_texture_material(
    path: &str,
    resources: &Resources,
    shader: Handle<Shader>,
    color: Color,
) -> Handle<Material> {
    let texture = resources.get::<AssetStore<Texture>>().load(
        path,
        koi_graphics_context::TextureSettings {
            minification_filter: FilterMode::Nearest,
            magnification_filter: FilterMode::Nearest,
            ..koi_graphics_context::TextureSettings::default()
        },
    );

    resources.get::<AssetStore<Material>>().add(Material {
        shader,
        base_color_texture: Some(texture),
        base_color: color,
        ..Default::default()
    })
}
