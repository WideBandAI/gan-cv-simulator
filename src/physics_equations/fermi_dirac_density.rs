//! Local 3D parabolic-band carrier statistics (not a quantized 2DEG model).
use crate::constants::physics::{K_BOLTZMANN, Q_ELECTRON};
use crate::physics_equations::TemperatureAware;
use crate::physics_equations::band_density::ConductionBandDensity;
use crate::physics_equations::electron_density::{BoltzmannApproximation, ElectronDensity};
use crate::physics_equations::equilibrium_potential::equilibrium_potential_n_type;
use std::f64::consts::PI;

/// Selects the conduction-band electron density model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CarrierStatistics {
    /// Legacy nondegenerate approximation.
    #[default]
    Boltzmann,
    /// Energy-integrated Fermi-Dirac statistics for a 3D parabolic band.
    FermiDirac,
}

impl CarrierStatistics {
    /// Construct the selected model at `temperature` in kelvin.
    pub fn model(self, temperature: f64) -> Box<dyn ElectronDensity> {
        match self {
            Self::Boltzmann => Box::new(BoltzmannApproximation::new(temperature)),
            Self::FermiDirac => Box::new(FermiDiracElectronDensity::new(temperature)),
        }
    }

    /// Return Ec-Ef in eV for n=Nd, assuming complete donor ionization.
    /// Explicit boundary values are not recalculated by this helper.
    /// Returns NaN for invalid physical inputs.
    pub fn equilibrium_potential(self, mass: f64, donor_density: f64, temperature: f64) -> f64 {
        if !mass.is_finite()
            || mass <= 0.0
            || !donor_density.is_finite()
            || donor_density <= 0.0
            || !temperature.is_finite()
            || temperature <= 0.0
        {
            return f64::NAN;
        }
        if self == Self::Boltzmann {
            return equilibrium_potential_n_type(mass, donor_density, temperature);
        }
        let nc = ConductionBandDensity::new(temperature).conduction_band_density(mass);
        let target = donor_density / nc;
        // In the dilute limit F_1/2(eta) ~ exp(eta); the next correction is negligible.
        if target < 1e-12 {
            return -(K_BOLTZMANN * temperature / Q_ELECTRON) * target.ln();
        }
        let mut lo = target.ln().min(-1.0) - 1.0;
        let mut hi = ((target * 0.75 * PI.sqrt()).powf(2.0 / 3.0) + 2.0).max(2.0);
        for _ in 0..100 {
            let mid = lo + (hi - lo) / 2.0;
            if fermi_dirac_half(mid) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        -(K_BOLTZMANN * temperature / Q_ELECTRON) * (lo + (hi - lo) / 2.0)
    }
}

/// Electron density n=Nc F_1/2(-(Ec-Ef)/(kBT/q)), in m^-3.
#[derive(Debug)]
pub struct FermiDiracElectronDensity {
    temperature: f64,
    conduction_band_density: ConductionBandDensity,
}

impl FermiDiracElectronDensity {
    /// Construct a density model at the specified temperature in kelvin.
    pub fn new(temperature: f64) -> Self {
        Self {
            temperature,
            conduction_band_density: ConductionBandDensity::new(temperature),
        }
    }
}

impl TemperatureAware for FermiDiracElectronDensity {
    fn set_temperature(&mut self, temperature: f64) {
        self.temperature = temperature;
        self.conduction_band_density.set_temperature(temperature);
    }
    fn get_temperature(&self) -> f64 {
        self.temperature
    }
}

impl ElectronDensity for FermiDiracElectronDensity {
    fn electron_density(&self, potential: f64, mass_electron: f64) -> f64 {
        if !self.temperature.is_finite()
            || self.temperature <= 0.0
            || !mass_electron.is_finite()
            || mass_electron < 0.0
            || !potential.is_finite()
        {
            return f64::NAN;
        }
        // Insulator mesh points have zero effective mass, as in the legacy model.
        if mass_electron == 0.0 {
            return 0.0;
        }
        let eta = -potential * (Q_ELECTRON / (K_BOLTZMANN * self.temperature));
        self.conduction_band_density
            .conduction_band_density(mass_electron)
            * fermi_dirac_half(eta)
    }
}

/// Gamma-normalized complete Fermi-Dirac integral of order 1/2.
///
/// Evaluates (2/sqrt(pi)) integral_0^infinity sqrt(x)/(1+exp(x-eta)) dx.
/// Uses x=u^2 and composite 16-point Gauss-Legendre quadrature, with u panels
/// at most 0.5 wide. The tail starts at x=max(eta,0)+48; its bound is
/// 2/sqrt(pi)*exp(eta-X)*(sqrt(X)+1/(2sqrt(X))). For eta<=0 the integrand
/// is scaled by exp(-eta), preserving relative accuracy in depleted regions.
/// For eta>64 a Sommerfeld expansion through relative order eta^-6 avoids
/// unbounded work in extreme accumulation. Returns NaN for NaN input.
pub fn fermi_dirac_half(eta: f64) -> f64 {
    if eta.is_nan() {
        return f64::NAN;
    }
    if eta > 64.0 {
        let r = (PI / eta).powi(2);
        return (4.0 / (3.0 * PI.sqrt()))
            * eta.powf(1.5)
            * (1.0 + r / 8.0 + 7.0 * r * r / 640.0 + 31.0 * r * r * r / 3072.0);
    }
    // Positive nodes/weights of the 16-point Gauss-Legendre rule on [-1,1].
    const NODES: [f64; 8] = [
        0.09501250983763744,
        0.2816035507792589,
        0.4580167776572274,
        0.6178762444026438,
        0.755404408355003,
        0.8656312023878318,
        0.9445750230732326,
        0.9894009349916499,
    ];
    const WEIGHTS: [f64; 8] = [
        0.1894506104550685,
        0.1826034150449236,
        0.16915651939500254,
        0.14959598881657673,
        0.12462897125553387,
        0.09515851168249279,
        0.06225352393864789,
        0.027152459411754095,
    ];
    let upper = (eta.max(0.0) + 48.0).sqrt();
    let panels = (upper / 0.5).ceil() as usize;
    let width = upper / panels as f64;
    let integrand = |u: f64| {
        let x = u * u;
        if eta <= 0.0 {
            x * (-x).exp() / (1.0 + (eta - x).exp())
        } else {
            let z = x - eta;
            let occupation = if z >= 0.0 {
                let e = (-z).exp();
                e / (1.0 + e)
            } else {
                1.0 / (1.0 + z.exp())
            };
            x * occupation
        }
    };
    let mut integral = 0.0;
    for panel in 0..panels {
        let center = (panel as f64 + 0.5) * width;
        for (&node, &weight) in NODES.iter().zip(WEIGHTS.iter()) {
            let offset = node * width / 2.0;
            integral += weight * (integrand(center - offset) + integrand(center + offset));
        }
    }
    let scale = if eta <= 0.0 { eta.exp() } else { 1.0 };
    4.0 / PI.sqrt() * integral * width / 2.0 * scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::physics::M_ELECTRON;
    use approx::assert_relative_eq;
    use test_case::test_case;

    // Independent 40-digit Decimal composite-Simpson reference values (8192 panels).
    #[test_case(-20.0, 2.0611536209365378e-9)]
    #[test_case(-10.0, 4.539920105264133e-5)]
    #[test_case(-2.0, 0.1292985133200756)]
    #[test_case(0.0, 0.765147024625408)]
    #[test_case(2.0, 2.823721277401584)]
    #[test_case(10.0, 24.08465696463765)]
    #[test_case(40.0, 190.4533903756893)]
    #[test_case(64.0, 385.2694537151697)]
    #[test_case(65.0, 394.33076085440764)]
    #[test_case(100.0, 752.3455915521961)]
    fn reference_integral(eta: f64, expected: f64) {
        assert_relative_eq!(fermi_dirac_half(eta), expected, max_relative = 1e-8);
    }

    #[test]
    fn limits_and_monotonicity() {
        assert_eq!(fermi_dirac_half(f64::NEG_INFINITY), 0.0);
        assert_eq!(fermi_dirac_half(f64::INFINITY), f64::INFINITY);
        assert!(fermi_dirac_half(f64::NAN).is_nan());
        assert_relative_eq!(
            fermi_dirac_half(-700.0),
            (-700.0_f64).exp(),
            max_relative = 1e-12
        );
        let mut previous = 0.0;
        for i in -1000..2000 {
            let value = fermi_dirac_half(i as f64 / 10.0);
            assert!(value > previous);
            previous = value;
        }
        assert_relative_eq!(
            fermi_dirac_half(64.0),
            fermi_dirac_half(64.0 + 1e-8),
            max_relative = 1e-8
        );
    }

    #[test]
    fn density_scaling_and_temperature() {
        let mut model = FermiDiracElectronDensity::new(300.0);
        let mass = 0.2 * M_ELECTRON;
        let nc = ConductionBandDensity::new(300.0).conduction_band_density(mass);
        assert_relative_eq!(
            model.electron_density(0.0, mass),
            nc * 0.765147024625408,
            max_relative = 1e-10
        );
        assert_relative_eq!(
            model.electron_density(0.0, 4.0 * mass),
            8.0 * model.electron_density(0.0, mass),
            max_relative = 1e-12
        );
        let potential = 10.0 * K_BOLTZMANN * 300.0 / Q_ELECTRON;
        let boltzmann = BoltzmannApproximation::new(300.0);
        assert_relative_eq!(
            model.electron_density(potential, mass),
            boltzmann.electron_density(potential, mass),
            max_relative = 2e-5
        );
        model.set_temperature(400.0);
        assert_relative_eq!(
            model.electron_density(0.1, mass),
            FermiDiracElectronDensity::new(400.0).electron_density(0.1, mass),
            max_relative = 1e-12
        );
        assert_eq!(model.electron_density(0.0, 0.0), 0.0);
        assert!(model.electron_density(0.0, -mass).is_nan());
        assert!(
            FermiDiracElectronDensity::new(0.0)
                .electron_density(0.0, mass)
                .is_nan()
        );
    }

    #[test]
    fn equilibrium_inverts_selected_density() {
        for density in [1e17, 1e22, 1e25, 1e27] {
            for statistics in [CarrierStatistics::Boltzmann, CarrierStatistics::FermiDirac] {
                let potential = statistics.equilibrium_potential(0.2 * M_ELECTRON, density, 300.0);
                assert_relative_eq!(
                    statistics
                        .model(300.0)
                        .electron_density(potential, 0.2 * M_ELECTRON),
                    density,
                    max_relative = 1e-8
                );
            }
        }
        assert!(
            CarrierStatistics::FermiDirac
                .equilibrium_potential(1.0, 0.0, 300.0)
                .is_nan()
        );
    }
}
