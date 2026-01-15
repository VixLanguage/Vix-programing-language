#[derive(Debug, Clone)] 
pub struct ArchConfig {
    pub pointer_bits: usize,
    pub pointer_align: usize,
    pub int_bits: usize,
    pub long_bits: usize,
    pub target: String,
}

impl ArchConfig {
    pub fn x86_64() -> Self {
        Self {
            pointer_bits: 64,
            pointer_align: 8,
            int_bits: 32,
            long_bits: 64,
            target: "x86_64".to_string(),
        }
    }
    
    pub fn x86() -> Self {
        Self {
            pointer_bits: 32,
            pointer_align: 4,
            int_bits: 32,
            long_bits: 32,
            target: "x86".to_string(),
        }
    }
    
    pub fn arm64() -> Self {
        Self {
            pointer_bits: 64,
            pointer_align: 8,
            int_bits: 32,
            long_bits: 64,
            target: "aarch64".to_string(),
        }
    }
    
    pub fn alignment_for_bits(&self, bits: usize) -> usize {
        match bits {
            1..=8 => 1,
            9..=16 => 2,
            17..=32 => 4,
            33..=64 => 8,
            _ => 16,
        }
    }
    
    pub fn tag_bits_for_variants(&self, count: usize) -> usize {
        match count {
            0..=256 => 8,
            257..=65536 => 16,
            _ => 32,
        }
    }
}
