//! Microblock slicing rules (`CoverRecipe`) and the placement resolver (`CoverLib.getPlacement`).
//! Pure data: the engine layer supplies the crafting grid and a "can this cover go there" query.

use super::multipart::{cover_to_strip_mask, damage_to_cover_data, strip_to_cover_mask, CoverLib};

/// One crafting-grid cell as `CoverRecipe.findResult` sees it.
#[derive(Clone, Debug, PartialEq)]
pub enum GridItem {
    /// Any handsaw; `sharpness` is `ItemHandsaw.getSharpness` (iron 0, gem 1, diamond 2).
    Saw { sharpness: i32 },
    /// A cover plate item with its damage value (`type << 8 | material`).
    CoverPlate { damage: i32 },
    /// Any other item; `block`/`meta` is its block identity if it is one.
    Other { block: String, meta: u16 },
}

/// Recipe output.
#[derive(Clone, Debug, PartialEq)]
pub enum RecipeResult {
    /// `count` cover plates of damage `ty << 8 | mat`.
    Cover { count: i32, ty: i32, mat: u8 },
    /// The full source block of material `mat` (`CoverLib.getItemStack`).
    Block { mat: u8 },
}

fn cover(count: i32, ty: i32, mat: i32) -> Option<RecipeResult> {
    Some(RecipeResult::Cover {
        count,
        ty,
        mat: mat as u8,
    })
}

/// `getSawRecipe`. `sawpos`/`matpos` are `column + row * 16`.
fn saw_recipe(
    lib: &CoverLib,
    sharpness: i32,
    sawpos: i32,
    mat: &GridItem,
    matpos: i32,
) -> Option<RecipeResult> {
    let (sp1, sp2) = (sawpos & 15, sawpos >> 4);
    let (mp1, mp2) = (matpos & 15, matpos >> 4);
    let (mn, dmg) = match mat {
        GridItem::CoverPlate { damage } => (damage & 0xFF, damage >> 8),
        GridItem::Other { block, meta } => (lib.material_for(block, *meta)? as i32, -1),
        GridItem::Saw { .. } => return None,
    };
    if sharpness < lib.hardness(mn as u8) {
        return None;
    }
    if sp1 == mp1 && (sp2 == mp2 + 1 || sp2 == mp2 - 1) {
        // Saw above or below: halve the thickness.
        let ty = match dmg {
            -1 => 17,
            16 => 0,
            17 => 16,
            25 => 24,
            26 => 25,
            29 => 27,
            33 => 31,
            _ => return None,
        };
        cover(2, ty, mn)
    } else if sp2 == mp2 && (sp1 == mp1 + 1 || sp1 == mp1 - 1) {
        // Saw beside: slice into strips, strips into corners.
        let ty = match dmg {
            0 => 21,
            16 => 22,
            17 => 23,
            21 => 18,
            22 => 19,
            23 => 20,
            27 => 39,
            28 => 40,
            29 => 41,
            30 => 42,
            39 => 35,
            40 => 36,
            41 => 37,
            42 => 38,
            _ => return None,
        };
        cover(2, ty, mn)
    } else {
        None
    }
}

/// `getColumnRecipe`: posts <-> pillars <-> columns, without a saw.
fn column_recipe(mat: &GridItem) -> Option<RecipeResult> {
    let GridItem::CoverPlate { damage } = mat else {
        return None;
    };
    let mn = damage & 0xFF;
    let ty = match damage >> 8 {
        22 => 43,
        23 => 44,
        41 => 45,
        43 => 22,
        44 => 23,
        45 => 41,
        _ => return None,
    };
    cover(1, ty, mn)
}

/// `getMergeRecipe`: `mn` is the packed class of the pieces, `tth` their summed 1/8 thickness.
fn merge_recipe(mn: i32, tth: i32, ic: i32) -> Option<RecipeResult> {
    let mc = mn >> 20;
    let m = mn & 255;
    let block = Some(RecipeResult::Block { mat: m as u8 });
    match mc {
        0 => match tth {
            2 => cover(1, 16, m),
            3 => cover(1, 27, m),
            4 => cover(1, 17, m),
            5 => cover(1, 28, m),
            6 => cover(1, 29, m),
            7 => cover(1, 30, m),
            8 => block,
            _ => None,
        },
        1 => match tth {
            2 => cover(1, 25, m),
            3 => cover(1, 31, m),
            4 => cover(1, 26, m),
            5 => cover(1, 32, m),
            6 => cover(1, 33, m),
            7 => cover(1, 34, m),
            8 => block,
            _ => None,
        },
        16 => match tth {
            2 => cover(1, 0, m),
            4 => cover(1, 16, m),
            8 => cover(1, 17, m),
            16 => block,
            _ => None,
        },
        32 if ic == 2 => match tth {
            2 => cover(1, 21, m),
            4 => cover(1, 22, m),
            8 => cover(1, 23, m),
            _ => None,
        },
        32 => match tth {
            4 => cover(1, 0, m),
            8 => cover(1, 16, m),
            16 => cover(1, 17, m),
            32 => block,
            _ => None,
        },
        _ => None,
    }
}

/// `getHollowRecipe`: eight covers around an empty centre make eight hollow covers.
fn hollow_recipe(mn: i32) -> Option<RecipeResult> {
    let ty = match (mn >> 8) & 0xFF {
        0 => 24,
        16 => 25,
        17 => 26,
        27 => 31,
        28 => 32,
        29 => 33,
        30 => 34,
        _ => return None,
    };
    cover(8, ty, mn & 255)
}

/// `getMicroClass`: packed class data of a cover plate, -1 for anything else.
fn micro_class(it: &GridItem) -> i32 {
    match it {
        GridItem::CoverPlate { damage } => damage_to_cover_data(*damage),
        _ => -1,
    }
}

/// `CoverRecipe.findResult` over the 3x3 grid, indexed `grid[column][row]`.
pub fn find_result(lib: &CoverLib, grid: &[[Option<GridItem>; 3]; 3]) -> Option<RecipeResult> {
    let mut saw: Option<(i32, i32)> = None; // (sharpness, pos)
    let mut mat: Option<(&GridItem, i32)> = None;
    let (mut bad, mut allmicro, mut strict) = (false, true, true);
    let (mut mn, mut tth, mut ic) = (-1i32, 0i32, 0i32);
    for (i, col) in grid.iter().enumerate() {
        for (j, cell) in col.iter().enumerate() {
            let Some(it) = cell else { continue };
            let pos = i as i32 + j as i32 * 16;
            if let GridItem::Saw { sharpness } = it {
                if saw.is_some() {
                    bad = true;
                } else {
                    saw = Some((*sharpness, pos));
                }
            } else if mat.is_none() {
                mat = Some((it, pos));
                mn = micro_class(it);
                if mn >= 0 {
                    tth += mn >> 16 & 15;
                } else {
                    allmicro = false;
                }
                ic = 1;
            } else {
                bad = true;
                if allmicro {
                    let t = micro_class(it);
                    if (t ^ mn) & -1048321 != 0 {
                        allmicro = false;
                    } else {
                        if t != mn {
                            strict = false;
                        }
                        tth += t >> 16 & 15;
                        ic += 1;
                    }
                }
            }
        }
    }
    match (saw, mat) {
        (Some((sh, sp)), Some((m, mp))) if !bad => saw_recipe(lib, sh, sp, m, mp),
        (None, Some((m, _))) if !bad => column_recipe(m),
        _ if !allmicro || !bad || saw.is_some() => None,
        _ => {
            if ic == 8 && strict && grid[1][1].is_none() && mn >> 20 == 0 {
                hollow_recipe(mn)
            } else {
                merge_recipe(mn, tth, ic)
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Placement
// ---------------------------------------------------------------------------------------------

/// A clicked position (`pl` / `MovingObjectPosition`): block, face `side`, absolute hit point,
/// and `sub_hit` the cover slot hit (-1 none).
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub pos: [i32; 3],
    pub side: i32,
    pub hit: [f64; 3],
    pub sub_hit: i32,
}

fn step_dir(p: &mut Placement) {
    match p.side {
        0 => p.pos[1] -= 1,
        1 => p.pos[1] += 1,
        2 => p.pos[2] -= 1,
        3 => p.pos[2] += 1,
        4 => p.pos[0] -= 1,
        _ => p.pos[0] += 1,
    }
}

/// `extractCoverSide`: which face the click was nearest on the clicked face's plane.
fn extract_cover_side(src: &Placement) -> i32 {
    let rx = src.hit[0] - src.pos[0] as f64 - 0.5;
    let ry = src.hit[1] - src.pos[1] as f64 - 0.5;
    let rz = src.hit[2] - src.pos[2] as f64 - 0.5;
    let sbw = 0.25;
    let centre = |a: f64, b: f64| a > -sbw && a < sbw && b > -sbw && b < sbw;
    match src.side {
        0 | 1 => {
            if centre(rz, rx) {
                src.side
            } else if rz > rx {
                if rz > -rx {
                    3
                } else {
                    4
                }
            } else if rz > -rx {
                5
            } else {
                2
            }
        }
        2 | 3 => {
            if centre(ry, rx) {
                src.side
            } else if ry > rx {
                if ry > -rx {
                    1
                } else {
                    4
                }
            } else if ry > -rx {
                5
            } else {
                0
            }
        }
        4 | 5 => {
            if centre(ry, rz) {
                src.side
            } else if ry > rz {
                if ry > -rz {
                    1
                } else {
                    2
                }
            } else if ry > -rz {
                3
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// `isClickOutside`: was the click on the outer face of the existing part (so we place beyond it)?
fn is_click_outside(m: &Placement) -> bool {
    let sh = m.sub_hit;
    if sh < 0 {
        true
    } else if sh < 6 {
        m.side != (sh ^ 1)
    } else if sh < 14 {
        let fc = sh - 6;
        let fc = fc >> 2 | (fc & 3) << 1;
        (m.side ^ (fc >> (m.side >> 1))) & 1 == 0
    } else if sh < 26 {
        let fc = strip_to_cover_mask((sh - 14) as usize) as i32;
        fc & (1 << (m.side ^ 1)) <= 0
    } else {
        true
    }
}

/// `CoverLib.getPlacement`. `item` is the cover plate damage;
/// `can_add(pos, side, sub_hit, cover)` answers `canAddCover` (true for an empty replaceable
/// block, else the tile's `can_add_cover`).
pub fn get_placement(
    src: &Placement,
    item: i32,
    can_add: &dyn Fn([i32; 3], i32, i32, i32) -> bool,
) -> Option<Placement> {
    let mut tr = src.clone();
    let cval = damage_to_cover_data(item) & 65535;
    let ok = |p: &Placement| can_add(p.pos, p.side, p.sub_hit, cval);
    let outside = is_click_outside(src);
    match item >> 8 {
        0 | 16 | 17 | 24..=34 => {
            let dirx = extract_cover_side(src);
            if dirx != tr.side {
                tr.sub_hit = dirx;
                if !outside && ok(&tr) {
                    return Some(tr);
                }
                step_dir(&mut tr);
                return ok(&tr).then_some(tr);
            }
            if !outside {
                tr.sub_hit = dirx ^ 1;
                if ok(&tr) {
                    return Some(tr);
                }
            }
            tr.sub_hit = dirx;
            if ok(&tr) {
                Some(tr)
            } else if !outside {
                None
            } else {
                step_dir(&mut tr);
                tr.sub_hit = dirx ^ 1;
                ok(&tr).then_some(tr)
            }
        }
        18..=20 | 35..=38 => {
            let rpx = src.hit[0] - src.pos[0] as f64;
            let rpy = src.hit[1] - src.pos[1] as f64;
            let rpz = src.hit[2] - src.pos[2] as f64;
            let mut v = 0;
            if rpz > 0.5 {
                v += 1;
            }
            if rpx > 0.5 {
                v += 2;
            }
            if rpy > 0.5 {
                v += 4;
            }
            v = match src.side {
                0 => v & 3,
                1 => v | 4,
                2 => v & 6,
                3 => v | 1,
                4 => v & 5,
                _ => v | 2,
            };
            let dir2 = v ^ match src.side {
                0 | 1 => 4,
                2 | 3 => 1,
                _ => 2,
            };
            if outside {
                tr.sub_hit = dir2 + 6;
                step_dir(&mut tr);
                ok(&tr).then_some(tr)
            } else {
                tr.sub_hit = dir2 + 6;
                if ok(&tr) {
                    return Some(tr);
                }
                tr.sub_hit = v + 6;
                ok(&tr).then_some(tr)
            }
        }
        21..=23 | 39..=42 => {
            let d = extract_cover_side(src);
            if d == tr.side {
                return None;
            }
            let csm = cover_to_strip_mask(d as usize);
            let strip =
                |side: i32| 14 + (csm & cover_to_strip_mask(side as usize)).trailing_zeros() as i32;
            if !outside {
                tr.sub_hit = strip(tr.side ^ 1);
                if ok(&tr) {
                    return Some(tr);
                }
                tr.sub_hit = strip(tr.side);
                ok(&tr).then_some(tr)
            } else {
                step_dir(&mut tr);
                tr.sub_hit = strip(tr.side ^ 1);
                ok(&tr).then_some(tr)
            }
        }
        43..=45 => {
            let d = extract_cover_side(src);
            if d != tr.side && d != (tr.side ^ 1) {
                return None;
            }
            if outside {
                step_dir(&mut tr);
            }
            tr.sub_hit = (d >> 1) + 26;
            ok(&tr).then_some(tr)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::multipart::CoverMaterial;

    fn mat(block: &str, hardness: i32) -> CoverMaterial {
        CoverMaterial {
            block: block.into(),
            meta: 0,
            name: String::new(),
            desc: String::new(),
            hardness,
            transparent: false,
            mining_hardness: 1.5,
        }
    }
    fn lib() -> CoverLib {
        let mut l = CoverLib::default();
        l.add_material(0, mat("minecraft:stone", 0));
        l.add_material(1, mat("minecraft:iron_block", 1));
        l
    }
    fn g() -> [[Option<GridItem>; 3]; 3] {
        Default::default()
    }
    fn stone() -> GridItem {
        GridItem::Other {
            block: "minecraft:stone".into(),
            meta: 0,
        }
    }
    fn plate(ty: i32) -> Option<GridItem> {
        Some(GridItem::CoverPlate { damage: ty << 8 })
    }
    fn res(count: i32, ty: i32) -> Option<RecipeResult> {
        Some(RecipeResult::Cover { count, ty, mat: 0 })
    }

    #[test]
    fn saw_halves_block_vertically() {
        let mut grid = g();
        grid[0][0] = Some(GridItem::Saw { sharpness: 0 });
        grid[0][1] = Some(stone());
        assert_eq!(find_result(&lib(), &grid), res(2, 17));
    }

    #[test]
    fn hardness_gates_saw() {
        let mut grid = g();
        grid[0][0] = Some(GridItem::Saw { sharpness: 0 });
        grid[0][1] = Some(GridItem::Other {
            block: "minecraft:iron_block".into(),
            meta: 0,
        });
        assert_eq!(find_result(&lib(), &grid), None);
        grid[0][0] = Some(GridItem::Saw { sharpness: 1 });
        assert!(find_result(&lib(), &grid).is_some());
    }

    #[test]
    fn saw_beside_makes_strips() {
        let mut grid = g();
        grid[0][0] = Some(GridItem::Saw { sharpness: 2 });
        grid[1][0] = plate(0);
        assert_eq!(find_result(&lib(), &grid), res(2, 21));
    }

    #[test]
    fn merge_covers() {
        let mut grid = g();
        grid[0][0] = plate(0);
        grid[1][0] = plate(0);
        assert_eq!(find_result(&lib(), &grid), res(1, 16));
        let mut grid = g();
        grid[0][0] = plate(17);
        grid[1][0] = plate(17);
        assert_eq!(
            find_result(&lib(), &grid),
            Some(RecipeResult::Block { mat: 0 })
        );
    }

    #[test]
    fn eight_covers_make_hollow() {
        let mut grid = g();
        for i in 0..3 {
            for j in 0..3 {
                if (i, j) != (1, 1) {
                    grid[i][j] = plate(0);
                }
            }
        }
        assert_eq!(find_result(&lib(), &grid), res(8, 24));
    }

    #[test]
    fn post_pillar_conversion() {
        let mut grid = g();
        grid[1][1] = plate(22);
        assert_eq!(find_result(&lib(), &grid), res(1, 43));
    }

    #[test]
    fn placement_face() {
        let src = Placement {
            pos: [0, 0, 0],
            side: 1,
            hit: [0.5, 1.0, 0.5],
            sub_hit: -1,
        };
        let r = get_placement(&src, 0, &|p, _, _, _| p != [0, 0, 0]).unwrap();
        assert_eq!((r.pos, r.sub_hit), ([0, 1, 0], 0));
        let src = Placement {
            hit: [0.95, 1.0, 0.5],
            ..src
        };
        let r = get_placement(&src, 0, &|p, _, _, _| p != [0, 0, 0]).unwrap();
        assert_eq!((r.pos, r.sub_hit), ([0, 1, 0], 5));
    }

    #[test]
    fn placement_corner_and_post() {
        let src = Placement {
            pos: [0, 0, 0],
            side: 1,
            hit: [0.9, 1.0, 0.9],
            sub_hit: -1,
        };
        let r = get_placement(&src, 18 << 8, &|_, _, _, _| true).unwrap();
        assert!((6..14).contains(&r.sub_hit));
        let src = Placement {
            hit: [0.5, 1.0, 0.5],
            ..src
        };
        let r = get_placement(&src, 43 << 8, &|_, _, _, _| true).unwrap();
        assert_eq!(r.sub_hit, 26);
        assert_eq!(get_placement(&src, 1 << 8, &|_, _, _, _| true), None);
    }
}
