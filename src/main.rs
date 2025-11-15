use rppal::gpio::Gpio;

fn main() -> rppal::gpio::Result<()> {
    let l_pin = Gpio::new()?.get(19)?.into_input();
    let m_pin = Gpio::new()?.get(16)?.into_input();
    let r_pin = Gpio::new()?.get(20)?.into_input();

    loop {
        let l_pin_level = l_pin.read();
        println!("LEFT LDR value: {:?}", l_pin_level);

        let m_pin_level = m_pin.read();
        println!("MIDDLE LDR value: {:?}", m_pin_level);

        let r_pin_level = r_pin.read();
        println!("RIGHT LDR value: {:?}", r_pin_level);

        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
