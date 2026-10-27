//! Waffen: Daten, Inventar, Eingaben, Schießen, Nahkampf, Projektile, Munition und
//! die Ego-Ansicht.

pub mod ammo;
pub mod assets;
pub mod defs;
pub mod firing;
pub mod input;
pub mod loadout;
pub mod melee;
pub mod projectiles;
pub mod viewmodel;

use bevy::prelude::*;

use crate::base::{AppState, GameSet, InGame};

pub use defs::{WEAPONS, WeaponClass, def, weapon_index};
pub use loadout::{
    GiveWeapon, Loadout, MAX_GRENADES, WState, WeaponSlot, enter_last_stand, leave_last_stand,
    take_current,
};
pub use projectiles::explode;
pub use viewmodel::VM_SCALE;

/// Alle Systeme, die Treffer erzeugen (Zombies werten danach den Schaden aus).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WeaponSystems;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GiveWeapon>()
            .add_systems(Startup, assets::setup_weapon_assets)
            .add_systems(
                OnEnter(AppState::Playing),
                (
                    loadout::spawn_loadout,
                    viewmodel::spawn_viewmodel,
                    viewmodel::spawn_bottle,
                )
                    .after(crate::player::spawn_player),
            )
            .add_systems(
                Update,
                (
                    input::weapon_input,
                    melee::melee_and_throw,
                    firing::fire_weapons,
                    projectiles::update_projectiles,
                    ammo::give_weapons,
                    ammo::max_ammo,
                )
                    .chain()
                    .in_set(WeaponSystems)
                    .in_set(GameSet::Combat)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            )
            .add_systems(
                Update,
                (
                    viewmodel::sync_viewmodel,
                    viewmodel::animate_viewmodel,
                    viewmodel::animate_bottle,
                )
                    .chain()
                    .in_set(GameSet::Ui)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            );
    }
}
