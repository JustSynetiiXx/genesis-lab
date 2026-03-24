#[derive(Clone, Debug)]
pub enum Topologie {
    Linear,
    Grid { breite: usize, hoehe: usize },
}

#[derive(Clone, Debug)]
pub enum NahrungModus {
    Gleichverteilt,
    Gradient { zentren: Vec<(usize, usize, f32)> },
}

#[derive(Clone, Debug)]
pub struct Config {
    pub speicher_groesse: usize,
    pub population_limit: usize,
    pub nahrung_wert: u8,
    pub nahrung_pro_tick: usize,
    pub nahrung_vorfuellung: f32,
    pub spawn_energie: i32,
    pub schritt_cap: usize,
    pub kopier_kosten_divisor: usize,
    pub mutationsrate: usize,
    pub verfall_pro_tick: usize,
    pub blitz_chance: usize,
    pub blitz_min_pop: usize,
    pub blitz_prozent: f32,
    pub leerlauf_tod_ticks: usize,
    pub min_kopier_groesse: usize,
    pub max_zellgroesse: usize,
    pub anweisung_groesse: usize,
    pub fress_energie: i32,
    pub topologie: Topologie,
    pub nahrung_modus: NahrungModus,
    pub nahrung_wert_b: u8,
    pub nahrung_b_anteil: f32,
    pub kopieren_braucht_b: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            speicher_groesse: 1_048_576,
            population_limit: 2000,
            nahrung_wert: 42,
            nahrung_pro_tick: 200,
            nahrung_vorfuellung: 0.2,
            spawn_energie: 30,
            schritt_cap: 500,
            kopier_kosten_divisor: 4,
            mutationsrate: 500,
            verfall_pro_tick: 100,
            blitz_chance: 3000,
            blitz_min_pop: 500,
            blitz_prozent: 0.07,
            leerlauf_tod_ticks: 10,
            min_kopier_groesse: 20,
            max_zellgroesse: 1024,
            anweisung_groesse: 4,
            fress_energie: 20,
            topologie: Topologie::Linear,
            nahrung_modus: NahrungModus::Gleichverteilt,
            nahrung_wert_b: 43,
            nahrung_b_anteil: 0.3,
            kopieren_braucht_b: true,
        }
    }
}

impl Config {
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let v: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON parse error: {}", e))?;
        let mut cfg = Config::default();

        if let Some(val) = v.get("speicher_groesse").and_then(|v| v.as_u64()) {
            cfg.speicher_groesse = val as usize;
        }
        if let Some(val) = v.get("population_limit").and_then(|v| v.as_u64()) {
            cfg.population_limit = val as usize;
        }
        if let Some(val) = v.get("nahrung_pro_tick").and_then(|v| v.as_u64()) {
            cfg.nahrung_pro_tick = val as usize;
        }
        if let Some(val) = v.get("spawn_energie").and_then(|v| v.as_i64()) {
            cfg.spawn_energie = val as i32;
        }
        if let Some(val) = v.get("schritt_cap").and_then(|v| v.as_u64()) {
            cfg.schritt_cap = val as usize;
        }
        if let Some(val) = v.get("mutationsrate").and_then(|v| v.as_u64()) {
            cfg.mutationsrate = val as usize;
        }
        if let Some(val) = v.get("verfall_pro_tick").and_then(|v| v.as_u64()) {
            cfg.verfall_pro_tick = val as usize;
        }
        if let Some(val) = v.get("blitz_chance").and_then(|v| v.as_u64()) {
            cfg.blitz_chance = val as usize;
        }
        if let Some(val) = v.get("fress_energie").and_then(|v| v.as_i64()) {
            cfg.fress_energie = val as i32;
        }
        if let Some(val) = v.get("nahrung_vorfuellung").and_then(|v| v.as_f64()) {
            cfg.nahrung_vorfuellung = val as f32;
        }
        if let Some(val) = v.get("kopieren_braucht_b").and_then(|v| v.as_bool()) {
            cfg.kopieren_braucht_b = val;
        }

        let mut use_grid = false;
        let mut grid_breite: usize = 1024;
        let mut grid_hoehe: usize = 1024;

        if let Some(topo) = v.get("topologie").and_then(|v| v.as_str()) {
            if topo == "grid" {
                use_grid = true;
            }
        }
        if let Some(val) = v.get("grid_breite").and_then(|v| v.as_u64()) {
            grid_breite = val as usize;
        }
        if let Some(val) = v.get("grid_hoehe").and_then(|v| v.as_u64()) {
            grid_hoehe = val as usize;
        }

        let mut use_gradient = false;
        if let Some(nm) = v.get("nahrung_modus").and_then(|v| v.as_str()) {
            if nm == "gradient" {
                use_gradient = true;
            }
        }

        if use_grid {
            cfg.topologie = Topologie::Grid { breite: grid_breite, hoehe: grid_hoehe };
            cfg.speicher_groesse = grid_breite * grid_hoehe;

            if use_gradient {
                let zentren = if let Some(arr) = v.get("oasen").and_then(|v| v.as_array()) {
                    arr.iter().filter_map(|o| {
                        let x = o.get(0).and_then(|v| v.as_u64())? as usize;
                        let y = o.get(1).and_then(|v| v.as_u64())? as usize;
                        let s = o.get(2).and_then(|v| v.as_f64())? as f32;
                        Some((x, y, s))
                    }).collect()
                } else {
                    vec![
                        (grid_breite / 4, grid_hoehe / 4, 3.0),
                        (grid_breite * 3 / 4, grid_hoehe / 4, 3.0),
                        (grid_breite / 4, grid_hoehe * 3 / 4, 3.0),
                        (grid_breite * 3 / 4, grid_hoehe * 3 / 4, 3.0),
                    ]
                };
                cfg.nahrung_modus = NahrungModus::Gradient { zentren };
            }
        }

        Ok(cfg)
    }

    pub fn set_parameter(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "population_limit" => { cfg_parse!(self.population_limit, value) }
            "nahrung_pro_tick" => { cfg_parse!(self.nahrung_pro_tick, value) }
            "spawn_energie" => { cfg_parse!(self.spawn_energie, value) }
            "schritt_cap" => { cfg_parse!(self.schritt_cap, value) }
            "mutationsrate" => { cfg_parse!(self.mutationsrate, value) }
            "verfall_pro_tick" => { cfg_parse!(self.verfall_pro_tick, value) }
            "blitz_chance" => { cfg_parse!(self.blitz_chance, value) }
            "fress_energie" => { cfg_parse!(self.fress_energie, value) }
            "nahrung_vorfuellung" => { cfg_parse_f32!(self.nahrung_vorfuellung, value) }
            "blitz_prozent" => { cfg_parse_f32!(self.blitz_prozent, value) }
            "kopieren_braucht_b" => {
                self.kopieren_braucht_b = value.parse().map_err(|e| format!("{}", e))?;
                Ok(())
            }
            _ => Err(format!("Unknown parameter: {}", key)),
        }
    }

    #[inline]
    pub fn grid_dims(&self) -> Option<(usize, usize)> {
        match self.topologie {
            Topologie::Grid { breite, hoehe } => Some((breite, hoehe)),
            _ => None,
        }
    }

    pub fn oasen(&self) -> Option<&Vec<(usize, usize, f32)>> {
        match &self.nahrung_modus {
            NahrungModus::Gradient { zentren } => Some(zentren),
            _ => None,
        }
    }
}

macro_rules! cfg_parse {
    ($field:expr, $value:expr) => {{
        $field = $value.parse().map_err(|e| format!("{}", e))?;
        Ok(())
    }};
}
pub(crate) use cfg_parse;

macro_rules! cfg_parse_f32 {
    ($field:expr, $value:expr) => {{
        $field = $value.parse::<f32>().map_err(|e| format!("{}", e))?;
        Ok(())
    }};
}
pub(crate) use cfg_parse_f32;
