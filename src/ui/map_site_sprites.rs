//! Textured landmark sprites layered over the strategic map silhouettes.

use super::map_sites::MarkerKind;
use super::{MapSpriteTextures, UiContext};
use crate::data::SiteDef;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
struct SpriteSpec<'a> {
    texture: &'a Texture2D,
    width: f32,
    height: f32,
    anchor: f32,
}

pub(super) fn draw_site_sprite(
    ctx: &UiContext<'_>,
    site: &SiteDef,
    kind: MarkerKind,
    position: Vec2,
    scale: f32,
) -> bool {
    let Some(spec) = sprite_spec(&ctx.sprites, site, kind) else {
        return false;
    };

    let width = spec.width * scale;
    let height = spec.height * scale;
    draw_texture_ex(
        spec.texture,
        position.x - width * 0.5,
        position.y + 7.0 * scale - height * spec.anchor,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width, height)),
            ..Default::default()
        },
    );
    true
}

pub(super) fn hit_radius(
    textures: &MapSpriteTextures<'_>,
    site: &SiteDef,
    kind: MarkerKind,
    scale: f32,
    marker_radius: f32,
) -> f32 {
    let Some(spec) = sprite_spec(textures, site, kind) else {
        return marker_radius * scale + 9.0;
    };
    (spec.width.max(spec.height) * scale * 0.42).max(marker_radius * scale) + 10.0
}

fn sprite_spec<'a>(
    textures: &MapSpriteTextures<'a>,
    site: &SiteDef,
    kind: MarkerKind,
) -> Option<SpriteSpec<'a>> {
    let (texture, width, height, anchor) = match kind {
        MarkerKind::Capital => (textures.capital?, 112.0, 122.0, 0.92),
        MarkerKind::PlayerSettlement | MarkerKind::Settlement | MarkerKind::Independent => {
            (textures.village?, 96.0, 88.0, 0.92)
        }
        MarkerKind::Landmark if site.site_type == "ruin" => (textures.ruin?, 88.0, 102.0, 0.93),
        MarkerKind::Landmark if site.site_type == "resource" => {
            if site.id == "ironroot_grove" {
                (textures.grove?, 116.0, 112.0, 0.92)
            } else {
                (textures.resource?, 92.0, 86.0, 0.92)
            }
        }
        MarkerKind::Landmark if site.site_type == "ford" => {
            (textures.bridge?, 116.0, 88.0, 0.80)
        }
        _ => return None,
    };
    Some(SpriteSpec {
        texture,
        width,
        height,
        anchor,
    })
}
