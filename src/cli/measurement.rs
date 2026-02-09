use crate::utils::{get_parsed_input, get_parsed_input_with_default};

#[derive(Debug)]
pub struct Measurement {
    pub temperature: Temperature,
    pub voltage: Voltage,
    pub time: Time,
    pub stress: Stress,
}

#[derive(Debug)]
pub struct Stress {
    pub stress_voltage: f64,
    pub stress_relief_voltage: f64,
    pub stress_relief_time: f64,
}

#[derive(Debug)]
pub struct Temperature {
    pub temperature: f64,
}

#[derive(Debug)]
pub struct Voltage {
    pub start: f64,
    pub stop: f64,
    pub step: f64,
}

#[derive(Debug)]
pub struct Time {
    pub measurement_time: f64,
}

pub fn define_measurement() -> Measurement {
    println!("Define measurement.");
    let temperature = loop {
        let temperature: f64 =
            get_parsed_input_with_default("Enter the temperature (in K). Default is 300: ", 300.0);
        if temperature <= 0.0 {
            println!("Temperature cannot be less than or equal to zero. Please try again.");
        } else {
            break temperature;
        }
    };
    let voltage_start: f64 = get_parsed_input("Enter the starting voltage (in V): ");
    let voltage_stop: f64 = get_parsed_input("Enter the stopping voltage (in V): ");
    let voltage_step = loop {
        let voltage_step: f64 = get_parsed_input("Enter the voltage step (in V): ");
        if voltage_step == 0.0 {
            println!("Voltage step cannot be zero. Please try again.");
        } else {
            break voltage_step;
        }
    };
    let measurement_time: f64 =
        get_parsed_input_with_default("Enter the measurement time (in s): default is 100 ", 100.0);
    let voltage_stress: f64 =
        get_parsed_input_with_default("Enter the stress voltage (in V): default is 0 ", 0.0);
    let stress_relief_voltage: f64 =
        get_parsed_input_with_default("Enter the stress relief voltage (in V): default is 0 ", 0.0);
    let stress_relief_time: f64 =
        get_parsed_input_with_default("Enter the stress relief time (in s): default is 0 ", 0.0);

    Measurement {
        temperature: Temperature { temperature },
        voltage: Voltage {
            start: voltage_start,
            stop: voltage_stop,
            step: voltage_step,
        },
        time: Time { measurement_time },
        stress: Stress {
            stress_voltage: voltage_stress,
            stress_relief_voltage,
            stress_relief_time,
        },
    }
}
