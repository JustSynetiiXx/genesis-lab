mod config;
mod interpreter;
mod welt;

use std::collections::{HashMap, HashSet};

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde_json::json;
use wasm_bindgen::prelude::*;

use config::Config;
use interpreter::{abiogenese, abiogenese_grid_mitte, abiogenese_near_oase, Pointer};
use welt::Welt;

struct Simulation {
    welt: Welt,
    cfg: Config,
    rng: SmallRng,
    pointer_liste: Vec<Pointer>,
    belegte_adressen: HashSet<usize>,
    tick_nr: u64,
    ops_zaehler: [u64; 11],
}

impl Simulation {
    fn new(cfg: Config) -> Self {
        let mut rng = SmallRng::from_entropy();
        let mut welt = Welt::new(cfg.clone());

        welt.vorfuellen(&mut rng);

        let start_adr = match (cfg.grid_dims(), cfg.oasen()) {
            (Some((breite, hoehe)), Some(oasen)) => {
                abiogenese_near_oase(&mut welt, &mut rng, oasen, breite, hoehe)
            }
            (Some((breite, hoehe)), None) => abiogenese_grid_mitte(&mut welt, breite, hoehe),
            _ => abiogenese(&mut welt, &mut rng),
        };

        let mut belegte_adressen = HashSet::new();
        belegte_adressen.insert(start_adr);

        Simulation {
            welt,
            cfg,
            rng,
            pointer_liste: vec![Pointer::new(start_adr)],
            belegte_adressen,
            tick_nr: 0,
            ops_zaehler: [0; 11],
        }
    }

    fn tick(&mut self) -> String {
        self.tick_nr += 1;

        // 1. Nahrung streuen
        self.welt.nahrung_streuen(&mut self.rng);

        // 2. Verfall
        self.welt.verfall(&mut self.rng);

        // 3. Blitz
        if self.pointer_liste.len() > self.cfg.blitz_min_pop
            && self.rng.gen_range(1..=self.cfg.blitz_chance) == 1
        {
            self.welt.blitz(&mut self.rng);
        }

        // 4. Tick für jeden Pointer
        let mut alle_neuen: Vec<usize> = Vec::new();
        for p in self.pointer_liste.iter_mut() {
            p.tick(&mut self.welt, &mut self.rng, &mut self.ops_zaehler);
            for &adr in &p.neue_pointer {
                alle_neuen.push(adr);
            }
            p.neue_pointer.clear();
            p.mutationen.clear();
            if p.aktiv && p.leerlauf_ticks >= self.cfg.leerlauf_tod_ticks {
                p.aktiv = false;
            }
        }

        // 5. Neue Pointer hinzufügen
        let platz_frei = self.cfg.population_limit.saturating_sub(self.pointer_liste.len());
        let mut hinzugefuegt = 0;
        for adr in alle_neuen {
            if hinzugefuegt >= platz_frei {
                break;
            }
            if self.belegte_adressen.contains(&adr) {
                continue;
            }
            self.pointer_liste.push(Pointer::new(adr));
            self.belegte_adressen.insert(adr);
            hinzugefuegt += 1;
        }

        // 6. Inaktive entfernen
        let mut i = 0;
        while i < self.pointer_liste.len() {
            if !self.pointer_liste[i].aktiv {
                self.belegte_adressen.remove(&self.pointer_liste[i].startadresse);
                self.pointer_liste.swap_remove(i);
            } else {
                i += 1;
            }
        }

        // 7. Abiogenese
        if self.pointer_liste.is_empty() {
            let adr = abiogenese(&mut self.welt, &mut self.rng);
            self.pointer_liste.push(Pointer::new(adr));
            self.belegte_adressen.insert(adr);
        }

        // Status berechnen
        let pop = self.pointer_liste.len();
        let nahrung = self.welt.nahrung_zaehlen();
        let ops_total: u64 = self.ops_zaehler.iter().sum();
        let speicher_groesse = self.welt.groesse();

        // Diversität + Genom-Durchschnittslänge
        let g = speicher_groesse;
        let max_zell = self.cfg.max_zellgroesse;
        let mut genom_set: HashSet<u64> = HashSet::new();
        let mut laengen_summe: usize = 0;
        for p in &self.pointer_liste {
            let gl = genom_laenge(&self.welt.speicher, p.startadresse, g, max_zell);
            laengen_summe += gl;
            let hash = simple_hash(&self.welt.speicher, p.startadresse, gl, g);
            genom_set.insert(hash);
        }
        let diversitaet = genom_set.len();
        let genom_avg = if pop > 0 {
            (laengen_summe as f64 / pop as f64 * 10.0).round() / 10.0
        } else {
            0.0
        };
        let nahrung_prozent = (nahrung as f64 / speicher_groesse as f64 * 10000.0).round() / 100.0;

        let (grid_breite, grid_hoehe) = match self.cfg.grid_dims() {
            Some((b, h)) => (b as u64, h as u64),
            None => (0, 0),
        };

        let result = json!({
            "tick": self.tick_nr,
            "population": pop,
            "diversitaet": diversitaet,
            "nahrung": nahrung,
            "nahrung_prozent": nahrung_prozent,
            "ops": ops_total,
            "genom_avg": genom_avg,
            "speicher_groesse": speicher_groesse,
            "ops_verteilung": self.ops_zaehler,
            "grid_breite": grid_breite,
            "grid_hoehe": grid_hoehe,
        });

        result.to_string()
    }
}

fn genom_laenge(speicher: &[u8], startadresse: usize, groesse: usize, max_zell: usize) -> usize {
    for i in (0..max_zell).step_by(4) {
        let pos = (startadresse + i) % groesse;
        if speicher[pos] == 9 {
            return i + 4;
        }
    }
    max_zell
}

fn simple_hash(speicher: &[u8], start: usize, len: usize, groesse: usize) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for i in 0..len {
        h ^= speicher[(start + i) % groesse] as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

// Global simulation storage
use std::sync::Mutex;

static SIMULATIONS: Mutex<Option<HashMap<u32, Simulation>>> = Mutex::new(None);
static NEXT_ID: Mutex<u32> = Mutex::new(1);

fn with_sims<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<u32, Simulation>) -> R,
{
    let mut guard = SIMULATIONS.lock().unwrap();
    let sims = guard.get_or_insert_with(HashMap::new);
    f(sims)
}

#[wasm_bindgen]
pub fn neue_simulation(config_json: String) -> u32 {
    let cfg = match Config::from_json(&config_json) {
        Ok(c) => c,
        Err(_) => Config::default(),
    };

    let sim = Simulation::new(cfg);

    let mut id_guard = NEXT_ID.lock().unwrap();
    let id = *id_guard;
    *id_guard += 1;

    with_sims(|sims| {
        sims.insert(id, sim);
    });

    id
}

#[wasm_bindgen]
pub fn tick(handle_id: u32) -> String {
    with_sims(|sims| {
        match sims.get_mut(&handle_id) {
            Some(sim) => sim.tick(),
            None => json!({"error": "Invalid handle"}).to_string(),
        }
    })
}

#[wasm_bindgen]
pub fn get_weltkarte(handle_id: u32) -> Vec<u8> {
    with_sims(|sims| {
        match sims.get(&handle_id) {
            Some(sim) => sim.welt.speicher.clone(),
            None => Vec::new(),
        }
    })
}

#[wasm_bindgen]
pub fn set_parameter(handle_id: u32, key: String, value: String) {
    with_sims(|sims| {
        if let Some(sim) = sims.get_mut(&handle_id) {
            let _ = sim.cfg.set_parameter(&key, &value);
            sim.welt.config = sim.cfg.clone();
        }
    });
}
