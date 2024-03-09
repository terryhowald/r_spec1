#![allow(warnings)]
use std::thread;
use std::time::Duration;
use anyhow::Result;
use rppal::i2c::I2c;

// AS7341 I2C default slave address
const AS7341_ADDR: u16      = 0x39;
const AS7341_CONFIG: u8     = 0x70;
const AS7341_LED: u8        = 0x74;
const AS7341_ENABLE: u8     = 0x80;
const AS7341_ATIME: u8      = 0x81;
const AS7341_WTIME: u8      = 0x83;
const AS7341_CH0_DATA_L: u8 = 0x95;
const AS7341_CH0_DATA_H: u8 = 0x96;

const AS7341_STATUS_2: u8   = 0xA3;
const AS7341_CFG_0: u8      = 0xA9;
const AS7341_CFG_1: u8      = 0xAA;
const AS7341_CFG_6: u8      = 0xAF;
const AS7341_ASTEP_L: u8    = 0xCA;
const AS7341_ASTEP_H: u8    = 0xCB;

pub fn as7341_init(i2c: &mut I2c) -> Result<()> {
    // Set the I2C slave address to the device we're communicating with
    i2c.set_slave_address(AS7341_ADDR)?;   // Set AS7341 address to 0x39   

    Ok(())
}

// Enable AS7341
#[allow(dead_code)]
pub fn as7341_enable(i2c: &I2c, flag: bool) -> Result<()> {
    // Read from Enable register
    let mut data: u8 = i2c.smbus_read_byte(AS7341_ENABLE)?;

    // Check flag
    if flag == true {
        data = data | (1<<0);   // Set PON bit if true
    }
    else {
        data = data & (!1);       // Clear PON bit if false
    }

    // Write to Enable register to update PON bit
    i2c.smbus_write_byte(AS7341_ENABLE, 0x01)?; 

    i2c.smbus_write_byte(0x00,0x30)?;

    Ok(())
}

// Config AS7341
fn as7341_config(i2c: &I2c, mode: u8) -> Result<()> {
    // Set bank to 1
    as7341_set_bank(&i2c, 1)?;

    // Read Config register
    let mut data: u8 = i2c.smbus_read_byte(AS7341_CONFIG)?;

    // Set integration mode to SPM (0) mode
    data = (data & (!3)) | mode;

    // Write update Config register
    i2c.smbus_write_byte(AS7341_CONFIG, data)?;

    // Set bank to 0
    as7341_set_bank(&i2c, 0)?;

    Ok(())
}

// Set bank
fn as7341_set_bank(i2c: &I2c, bank: u8) -> Result<()> {
    // Check bank requested (0 or 1)
    if bank != 0 && bank != 1 {
        return Ok(());
    } 

    // Read from CFG0 register
    let mut data: u8 = i2c.smbus_read_byte(AS7341_CFG_0)?;

    // Set REG_BANK bit (4) with the bank to use
    if bank == 1 {
        data = data | (1<<4);
    }
    else {
        data = data & (!(1<<4));
    }

    // Write to CFG0 register to set active bank
    i2c.smbus_write_byte(AS7341_CFG_0, data)?;

    Ok(())
}

// Enable LEDs
pub fn as7341_enable_leds(i2c: &I2c, flag: bool) -> Result<()> {
    // Switch to Bank 1
    as7341_set_bank(&i2c, 0x01)?;

    let mut data: u8 = i2c.smbus_read_byte(AS7341_CONFIG)?;
    let mut data1: u8 = i2c.smbus_read_byte(AS7341_LED)?;
    if flag == true {
        data = data | 0x08;
    }
    else {
        data = data & 0xf7;
        data1 = data1 & 0x7f;
        i2c.smbus_write_byte(AS7341_LED, data)?;
    }
    i2c.smbus_write_byte(AS7341_CONFIG, data)?;

    // Switch to Bank 0
    as7341_set_bank(&i2c, 0x00)?;

    Ok(())
}

// Control LEDs
pub fn as7341_control_leds(i2c: &I2c, led: bool, mut current: u8) -> Result<()> {

    if current < 1 {
        current = 1;
    }
    current = current - 1;
    if current > 19 {
        current = 19;
    }

    // Switch to Bank 1
    as7341_set_bank(&i2c, 0x01)?;

    let mut data: u8 = 0x00;
    if led == true {
        data = 0x80 | current;
    }
    else {
        data = current;
    }
    i2c.smbus_write_byte(AS7341_LED, data)?;
    // thread::sleep(Duration::from_millis(100));

    // Switch to Bank 0
    as7341_set_bank(&i2c, 0)?;

    Ok(())
}

// Config ATIME component of integration time
pub fn as7341_atime_config(i2c: &I2c, value: u8) -> Result<()> {
    // Write to ATIME register
    i2c.smbus_write_byte(AS7341_ATIME,value)?;

    Ok(())
}

// Config ASTEP component of intergration time
pub fn as7341_astep_config(i2c: &I2c, value: u16) -> Result<()> {
    // Write to ASTEP registers        
    let low_value: u8 = (value & 0x00ff) as u8;
    let high_value: u8 = (value >> 8) as u8;       
    i2c.smbus_write_byte(AS7341_ASTEP_L, low_value)?;
    i2c.smbus_write_byte(AS7341_ASTEP_H, high_value)?;   
    
    Ok(())
}

// Config AGAIN
pub fn as7341_again_config(i2c: &I2c, mut value: u8) -> Result<()> {
    // Check value range
    if value > 10 {
        value = 10;
    }

    // Write to AGAIN register
    i2c.smbus_write_byte(AS7341_CFG_1, value)?;

    Ok(())
}

// Enable Spectral Measure
fn as7341_enable_spectral_measure(i2c: &I2c, flag: bool) -> Result<()> {
    // Read Enable register to access the SP_EN bit
    let mut data: u8 = i2c.smbus_read_byte(AS7341_ENABLE)?;

    // Toggle SP_EN bit based on flag
    if flag == true {
        data = data | (1<<1);
    }
    else {
        data = data & (!(1<<1));
    }

    // Write to Enable register with SP_EN bit configured
    i2c.smbus_write_byte(AS7341_ENABLE, data)?;

    Ok(())
}

// Clear F1-F4
fn as7341_f1f4_clear_nir(i2c: &I2c) -> Result<()> {
    // Clear spectral data for F1 through F4
    i2c.smbus_write_byte(0x00, 0x30)?; 
    i2c.smbus_write_byte(0x01, 0x01)?; 
    i2c.smbus_write_byte(0x02, 0x00)?; 
    i2c.smbus_write_byte(0x03, 0x00)?; 
    i2c.smbus_write_byte(0x04, 0x00)?; 
    i2c.smbus_write_byte(0x05, 0x42)?; 
    i2c.smbus_write_byte(0x06, 0x00)?; 
    i2c.smbus_write_byte(0x07, 0x00)?; 
    i2c.smbus_write_byte(0x08, 0x50)?; 
    i2c.smbus_write_byte(0x09, 0x00)?; 
    i2c.smbus_write_byte(0x0A, 0x00)?; 
    i2c.smbus_write_byte(0x0B, 0x00)?; 
    i2c.smbus_write_byte(0x0C, 0x20)?; 
    i2c.smbus_write_byte(0x0D, 0x04)?; 
    i2c.smbus_write_byte(0x0E, 0x00)?; 
    i2c.smbus_write_byte(0x0F, 0x30)?; 
    i2c.smbus_write_byte(0x10, 0x01)?; 
    i2c.smbus_write_byte(0x11, 0x50)?; 
    i2c.smbus_write_byte(0x12, 0x00)?; 
    i2c.smbus_write_byte(0x13, 0x06)?;     

    Ok(())
}

// Clear F5-F8
fn as7341_f5f8_clear_nir(i2c: &I2c) -> Result<()> {
    // Clear spectral data for F5 through F8
    i2c.smbus_write_byte(0x00, 0x00)?; 
    i2c.smbus_write_byte(0x01, 0x00)?; 
    i2c.smbus_write_byte(0x02, 0x00)?; 
    i2c.smbus_write_byte(0x03, 0x40)?; 
    i2c.smbus_write_byte(0x04, 0x02)?; 
    i2c.smbus_write_byte(0x05, 0x00)?; 
    i2c.smbus_write_byte(0x06, 0x10)?; 
    i2c.smbus_write_byte(0x07, 0x03)?; 
    i2c.smbus_write_byte(0x08, 0x50)?; 
    i2c.smbus_write_byte(0x09, 0x10)?; 
    i2c.smbus_write_byte(0x0A, 0x03)?; 
    i2c.smbus_write_byte(0x0B, 0x00)?; 
    i2c.smbus_write_byte(0x0C, 0x00)?; 
    i2c.smbus_write_byte(0x0D, 0x00)?; 
    i2c.smbus_write_byte(0x0E, 0x24)?; 
    i2c.smbus_write_byte(0x0F, 0x00)?; 
    i2c.smbus_write_byte(0x10, 0x00)?; 
    i2c.smbus_write_byte(0x11, 0x50)?; 
    i2c.smbus_write_byte(0x12, 0x00)?; 
    i2c.smbus_write_byte(0x13, 0x06)?; 

    Ok(())
}

// Enable SMUX
fn as7341_enable_smux(i2c: &I2c, flag: bool) -> Result<()> {
    // Read Enable register to get SMUX bit
    let mut data: u8 = i2c.smbus_read_byte(AS7341_ENABLE)?;

    // Set SMUX bit in Enable register
    if flag == true {
        data = data | (1<<4);
    }
    else {
        data = data & (!(1<<4));
    }

    // Write Enable register with updated SMUX bit
    i2c.smbus_write_byte(AS7341_ENABLE, data)?;

    Ok(())
}

// Measure complete
fn as7341_measure_complete(i2c: &I2c) -> bool {
    // Read status register
    let status: u8 = i2c.smbus_read_byte(AS7341_STATUS_2).unwrap();

    // Check measure status
    if (status & (1<<6)) > 0 {
        return true;
    }
    else {
        return false;
    }
}

// Start measure
pub fn as7341_start_measure(i2c: &I2c, mode: u8) -> Result<()> {
    // Check value range
    if mode != 0 && mode != 1 {
        return Ok(());
    }

    // Set bank to 0
    let mut data: u8 = i2c.smbus_read_byte(AS7341_CFG_0)?;
    data = data & (!(1<<4));
    i2c.smbus_write_byte(AS7341_CFG_0, data)?;

    // Disable spectral measure
    as7341_enable_spectral_measure(i2c, false)?;

    // Write SMUX_CMD bits
    i2c.smbus_write_byte(AS7341_CFG_6, 0x10)?;

    // Clear spectral data
    if mode == 0 {
        as7341_f1f4_clear_nir(&i2c)?;
    }
    else {
        as7341_f5f8_clear_nir(&i2c)?;
    }

    // Enable SMUX
    as7341_enable_smux(i2c, true)?;

    // Setup single spectral measure
    as7341_config(&i2c, 0)?;

    // Enable spectral measure
    as7341_enable_spectral_measure(&i2c, true)?;

    // Wait for measure to complete
    while as7341_measure_complete(&i2c) == false {
        thread::sleep(Duration::from_millis(100));  
    }    

    Ok(())
}

// Get Channel Data
pub fn as7341_get_channel_data(i2c: &I2c, chan: u8) -> u16 {
    // Set 16 bit variable to hold channel data
    let mut chan_data: u16 = 0;

    // Read channel low and high bytes
    let low_byte: u8 =  i2c.smbus_read_byte(AS7341_CH0_DATA_L + chan*2).unwrap();
    let high_byte: u8 = i2c.smbus_read_byte(AS7341_CH0_DATA_H + chan*2).unwrap();

    // Construct channel data
    chan_data = high_byte as u16;
    chan_data = (chan_data<<8) | low_byte as u16;

    return chan_data;
}

// Read Spectral Data One
pub fn as7341_read_spectral_data_one(i2c: &I2c) -> Result<()> {
    // Read channel data
    let chan1: u16 = as7341_get_channel_data(&i2c, 0);
    let chan2: u16 = as7341_get_channel_data(&i2c, 1);
    let chan3: u16 = as7341_get_channel_data(&i2c, 2);
    let chan4: u16 = as7341_get_channel_data(&i2c, 3);    

    //println!("Channel 1 = {}", chan1);
    //println!("Channel 2 = {}", chan2);
    //println!("Channel 3 = {}", chan3);
    //println!("Channel 4 = {}", chan4);    
//
    Ok(())
}

// Read Spectral Data Two
pub fn as7341_read_spectral_data_two(i2c: &I2c) -> Result<()> {
    // Read channel data
    let chan5: u16 = as7341_get_channel_data(&i2c, 0);
    let chan6: u16 = as7341_get_channel_data(&i2c, 1);
    let chan7: u16 = as7341_get_channel_data(&i2c, 2);
    let chan8: u16 = as7341_get_channel_data(&i2c, 3);    

    //println!("Channel 5 = {}", chan5);
    //println!("Channel 6 = {}", chan6);
    //println!("Channel 7 = {}", chan7);
    //println!("Channel 8 = {}", chan8);    

    Ok(())
}
