/// Transporte simulado. Uma implementação real pode substituir este contrato.
pub trait Audio {
    fn playing(&self) -> bool;
    fn set_playing(&mut self, value: bool);
    fn volume(&self) -> f32;
    fn set_volume(&mut self, value: f32);
    fn advance(&self, position: &mut f32, elapsed: f32);
}
pub struct SimulatedAudio {
    playing: bool,
    volume: f32,
}
impl Default for SimulatedAudio {
    fn default() -> Self {
        Self {
            playing: false,
            volume: 0.70,
        }
    }
}
impl Audio for SimulatedAudio {
    fn playing(&self) -> bool {
        self.playing
    }
    fn set_playing(&mut self, value: bool) {
        self.playing = value;
    }
    fn volume(&self) -> f32 {
        self.volume
    }
    fn set_volume(&mut self, value: f32) {
        self.volume = value.clamp(0.0, 1.0);
    }
    fn advance(&self, position: &mut f32, elapsed: f32) {
        if self.playing {
            *position += elapsed;
        }
    }
}
