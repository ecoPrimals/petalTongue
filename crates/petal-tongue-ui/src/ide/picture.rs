// SPDX-License-Identifier: AGPL-3.0-or-later
//! For tests: the window as a picture, without a display.
//!
//! egui hands every frame over as coloured, textured triangles. A desktop
//! gives them to the graphics card; here they are filled in one by one, in
//! plain code, into a grid of pixels. It is slow and it is exact enough to
//! look at: the same panels, the same words, the same colours.

use std::collections::HashMap;
use std::path::Path;

use egui::epaint::{ClippedPrimitive, ImageData, Mesh, Primitive, Vertex};
use egui::{Color32, Pos2, Rect, TextureId, TexturesDelta};

use super::state::State;
use super::view;

/// A texture as egui gave it: premultiplied colours, row after row.
struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<Color32>,
}

/// The pixels of one frame.
pub struct Picture {
    pub width: usize,
    pub height: usize,
    /// Red, green, blue, alpha for each pixel, row after row.
    pub rgba: Vec<u8>,
}

impl Picture {
    /// How many different colours are in it. A blank picture has one.
    pub fn colours(&self) -> usize {
        self.rgba
            .chunks_exact(4)
            .map(|p| [p[0], p[1], p[2]])
            .collect::<std::collections::HashSet<_>>()
            .len()
    }

    /// Write it as a PNG.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let size = tiny_skia::IntSize::from_wh(self.width as u32, self.height as u32)
            .ok_or("the picture has no size")?;
        tiny_skia::Pixmap::from_vec(self.rgba.clone(), size)
            .ok_or("the picture's pixels do not fit its size")?
            .save_png(path)
            .map_err(|e| e.to_string())
    }
}

/// Fills in what egui draws.
#[derive(Default)]
pub struct Painter {
    textures: HashMap<TextureId, Texture>,
}

impl Painter {
    /// Take in the textures egui made or changed this frame.
    pub fn textures(&mut self, delta: &TexturesDelta) {
        for (id, change) in &delta.set {
            let [width, height] = change.image.size();
            let pixels: Vec<Color32> = match &change.image {
                ImageData::Color(image) => image.pixels.clone(),
                ImageData::Font(image) => image.srgba_pixels(None).collect(),
            };
            match change.pos {
                None => {
                    self.textures.insert(
                        *id,
                        Texture {
                            width,
                            height,
                            pixels,
                        },
                    );
                }
                Some([left, top]) => {
                    let Some(whole) = self.textures.get_mut(id) else {
                        continue;
                    };
                    for row in 0..height {
                        let from = row * width;
                        let to = (top + row) * whole.width + left;
                        whole.pixels[to..to + width].copy_from_slice(&pixels[from..from + width]);
                    }
                }
            }
        }
        for id in &delta.free {
            self.textures.remove(id);
        }
    }

    /// Fill in one frame.
    pub fn paint(
        &self,
        width: usize,
        height: usize,
        scale: f32,
        background: Color32,
        primitives: &[ClippedPrimitive],
    ) -> Picture {
        let mut canvas = vec![
            [
                f32::from(background.r()),
                f32::from(background.g()),
                f32::from(background.b()),
            ];
            width * height
        ];
        for clipped in primitives {
            let Primitive::Mesh(mesh) = &clipped.primitive else {
                continue;
            };
            let Some(texture) = self.textures.get(&mesh.texture_id) else {
                continue;
            };
            let clip = Rect::from_min_max(
                (clipped.clip_rect.min.to_vec2() * scale).to_pos2(),
                (clipped.clip_rect.max.to_vec2() * scale).to_pos2(),
            )
            .intersect(Rect::from_min_max(
                Pos2::ZERO,
                Pos2::new(width as f32, height as f32),
            ));
            mesh_into(&mut canvas, width, mesh, texture, clip, scale);
        }
        let rgba = canvas
            .iter()
            .flat_map(|p| {
                [
                    p[0].round().clamp(0.0, 255.0) as u8,
                    p[1].round().clamp(0.0, 255.0) as u8,
                    p[2].round().clamp(0.0, 255.0) as u8,
                    255,
                ]
            })
            .collect();
        Picture {
            width,
            height,
            rgba,
        }
    }
}

fn mesh_into(
    canvas: &mut [[f32; 3]],
    width: usize,
    mesh: &Mesh,
    texture: &Texture,
    clip: Rect,
    scale: f32,
) {
    for corners in mesh.indices.chunks_exact(3) {
        let [a, b, c] = [
            mesh.vertices[corners[0] as usize],
            mesh.vertices[corners[1] as usize],
            mesh.vertices[corners[2] as usize],
        ];
        triangle_into(canvas, width, [a, b, c], texture, clip, scale);
    }
}

fn triangle_into(
    canvas: &mut [[f32; 3]],
    width: usize,
    [a, b, c]: [Vertex; 3],
    texture: &Texture,
    clip: Rect,
    scale: f32,
) {
    let at = |v: Vertex| (v.pos.x * scale, v.pos.y * scale);
    let ((ax, ay), (bx, by), (cx, cy)) = (at(a), at(b), at(c));
    let area = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
    if area.abs() < 1e-6 {
        return;
    }

    let left = ax.min(bx).min(cx).max(clip.min.x).floor().max(0.0) as usize;
    let top = ay.min(by).min(cy).max(clip.min.y).floor().max(0.0) as usize;
    let right = ax.max(bx).max(cx).min(clip.max.x).ceil().max(0.0) as usize;
    let bottom = ay.max(by).max(cy).min(clip.max.y).ceil().max(0.0) as usize;

    let colour = |v: Vertex| {
        [
            f32::from(v.color.r()),
            f32::from(v.color.g()),
            f32::from(v.color.b()),
            f32::from(v.color.a()),
        ]
    };
    let (ca, cb, cc) = (colour(a), colour(b), colour(c));

    for y in top..bottom {
        for x in left..right {
            // The middle of the pixel.
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            if px < clip.min.x || px >= clip.max.x || py < clip.min.y || py >= clip.max.y {
                continue;
            }
            let wa = ((bx - px) * (cy - py) - (by - py) * (cx - px)) / area;
            let wb = ((cx - px) * (ay - py) - (cy - py) * (ax - px)) / area;
            let wc = 1.0 - wa - wb;
            if wa < 0.0 || wb < 0.0 || wc < 0.0 {
                continue;
            }

            let across = wa * a.uv.x + wb * b.uv.x + wc * c.uv.x;
            let down = wa * a.uv.y + wb * b.uv.y + wc * c.uv.y;
            let tx = ((across * texture.width as f32) as usize).min(texture.width - 1);
            let ty = ((down * texture.height as f32) as usize).min(texture.height - 1);
            let texel = texture.pixels[ty * texture.width + tx];
            let texel = [
                f32::from(texel.r()),
                f32::from(texel.g()),
                f32::from(texel.b()),
                f32::from(texel.a()),
            ];

            // Colours are premultiplied: source over what is there.
            let mut source = [0.0_f32; 4];
            for channel in 0..4 {
                let tint = wa * ca[channel] + wb * cb[channel] + wc * cc[channel];
                source[channel] = tint * texel[channel] / 255.0;
            }
            let keep = 1.0 - source[3] / 255.0;
            let pixel = &mut canvas[y * width + x];
            for channel in 0..3 {
                pixel[channel] = source[channel] + pixel[channel] * keep;
            }
        }
    }
}

/// Draw the window for `state` as it would be after a few frames, at
/// `width` × `height` points. Returns the picture and whatever the view
/// asked for while drawing.
pub fn of(state: &mut State, width: f32, height: f32) -> (Picture, Vec<super::state::Action>) {
    of_with(state, width, height, |_| Vec::new())
}

/// As [`of`], with input events for each frame (`frame` counts from 0).
pub fn of_with(
    state: &mut State,
    width: f32,
    height: f32,
    events: impl Fn(usize) -> Vec<egui::Event>,
) -> (Picture, Vec<super::state::Action>) {
    let ctx = egui::Context::default();
    view::style(&ctx);
    let mut painter = Painter::default();
    let mut asked = Vec::new();
    let mut last = None;
    // A few frames: egui places some things only from the second frame on.
    for frame in 0..4 {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(width, height))),
            time: Some(frame as f64 / 60.0),
            events: events(frame),
            focused: true,
            ..Default::default()
        };
        let output = ctx.run(input, |ctx| asked.extend(view::draw(ctx, state)));
        painter.textures(&output.textures_delta);
        last = Some(output);
    }
    let output = last.expect("four frames were drawn");
    let scale = output.pixels_per_point;
    let background = ctx.style().visuals.panel_fill;
    let primitives = ctx.tessellate(output.shapes, scale);
    let picture = painter.paint(
        (width * scale) as usize,
        (height * scale) as usize,
        scale,
        background,
        &primitives,
    );
    (picture, asked)
}

/// Where pictures are kept when `GAIA_IDE_PICTURES` names a folder. Tests
/// save there so the window can be looked at; without it nothing is saved.
pub fn keep(picture: &Picture, name: &str) {
    let Some(folder) = std::env::var_os("GAIA_IDE_PICTURES") else {
        return;
    };
    let folder = Path::new(&folder);
    std::fs::create_dir_all(folder).expect("the folder for pictures can be made");
    picture
        .save(&folder.join(format!("{name}.png")))
        .expect("the picture can be saved");
}
