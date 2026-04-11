use crate::config::boundary_conditions::BoundaryConditions;
use crate::config::boundary_conditions::define_boundary_conditions;
use crate::config::capture_cross_section::CaptureCrossSectionConfig;
use crate::config::capture_cross_section::define_capture_cross_section;
use crate::config::fixcharge::BulkFixedCharge;
use crate::config::fixcharge::InterfaceFixedCharge;
use crate::config::fixcharge::define_bulk_fixed_charge;
use crate::config::fixcharge::define_interface_fixed_charge;
use crate::config::interface_states::ContinuousInterfaceStatesConfig;
use crate::config::interface_states::DiscreteInterfaceStatesConfig;
use crate::config::interface_states::define_interface_states;
use crate::config::measurement::Measurement;
use crate::config::measurement::define_measurement;
use crate::config::mesh::MeshParams;
use crate::config::mesh::define_mesh_params;
use crate::config::sim_settings::SimSettings;
use crate::config::sim_settings::define_sim_settings;
use crate::config::structure::DeviceStructure;
use crate::config::structure::define_structure;

#[derive(Debug, serde::Serialize)]
pub struct Configuration {
    pub measurement: Measurement,
    pub device_structure: DeviceStructure,
    pub bulk_fixed_charge: BulkFixedCharge,
    pub interface_fixed_charge: InterfaceFixedCharge,
    pub continuous_interface_states: ContinuousInterfaceStatesConfig,
    pub discrete_interface_states: DiscreteInterfaceStatesConfig,
    pub capture_cross_section: CaptureCrossSectionConfig,
    pub mesh_params: MeshParams,
    pub boundary_conditions: BoundaryConditions,
    pub sim_settings: SimSettings,
}

#[derive(Debug)]
pub struct ConfigurationBuilder {
    configuration: Configuration,
}

impl ConfigurationBuilder {
    /// Create a new ConfigurationBuilder from an existing Configuration
    pub fn new(configuration: Configuration) -> Self {
        Self { configuration }
    }

    /// Build configuration from interactive CLI input
    pub fn from_interactive() -> Self {
        println!("Starting C-V simulation with the following parameters:");
        let sim_settings = define_sim_settings();
        let measurement = define_measurement();
        let device_structure = define_structure();
        let bulk_fixed_charge = define_bulk_fixed_charge(&device_structure);
        let interface_fixed_charge = define_interface_fixed_charge(&device_structure);
        let (continuous_interface_states, discrete_interface_states) =
            define_interface_states(&device_structure);
        let capture_cross_section = define_capture_cross_section(
            &continuous_interface_states,
            &discrete_interface_states,
            &device_structure,
        );
        let mesh_params = define_mesh_params(&device_structure);
        let boundary_conditions = define_boundary_conditions(&device_structure, &measurement);

        let configuration = Configuration {
            measurement,
            device_structure,
            bulk_fixed_charge,
            interface_fixed_charge,
            continuous_interface_states,
            discrete_interface_states,
            capture_cross_section,
            mesh_params,
            boundary_conditions,
            sim_settings,
        };

        Self { configuration }
    }

    /// Build configuration from a JSON file (placeholder for future implementation)
    ///
    /// # Arguments
    /// * `path` - Path to the JSON configuration file
    ///
    /// # Example (future usage)
    /// ```ignore
    /// let builder = ConfigurationBuilder::from_json("config.json")?;
    /// ```
    #[allow(dead_code)]
    pub fn from_json(_path: &str) -> Result<Self, std::io::Error> {
        // TODO: Implement JSON deserialization
        // This is a placeholder for future implementation when serde support is added
        unimplemented!("JSON configuration loading is not yet implemented")
    }

    /// Get a reference to the configuration
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    /// Get a mutable reference to the configuration
    pub fn configuration_mut(&mut self) -> &mut Configuration {
        &mut self.configuration
    }

    /// Consume the builder and return the configuration
    pub fn build(self) -> Configuration {
        self.configuration
    }
}
