mod config;

use config::BluetoothConfig;

use crate::pinetimers::BluetoothTimer;

use nrf52832_hal::pac::{RADIO, FICR};

use rubble_nrf5x::utils::get_device_address;
use rubble_nrf5x::radio::{BleRadio, PacketBuffer};
use rubble_nrf5x::timer::BleTimer;
use rubble::link::queue::{SimpleQueue, PacketQueue};
use rubble::link::{LinkLayer, Responder, Cmd};
use rubble::l2cap::{L2CAPState, BleChannelMap};
use rubble::link::ad_structure::AdStructure;
use rubble::time::Timer;

use rubble::gatt::BatteryServiceAttrs;

pub struct Bluetooth {
    linklayer: LinkLayer<BluetoothConfig>,
    radio: BleRadio,
    responder: Responder<BluetoothConfig>,
}

impl Bluetooth {
    pub fn new(
        radio: RADIO,
        ficr: FICR,
        timer: BluetoothTimer,
        ble_tx_buf: &'static mut PacketBuffer,
        ble_rx_buf: &'static mut PacketBuffer,
        ble_tx_queue: &'static mut SimpleQueue,
        ble_rx_queue: &'static mut SimpleQueue,
    ) -> Bluetooth {
        let device_address = get_device_address();
        
        let mut ble_radio = BleRadio::new(
            radio,
            &ficr,
            ble_tx_buf,
            ble_rx_buf,
        );

        let ble_timer = BleTimer::init(timer);

        // Set up queues
        let (tx_prod, tx_cons) = ble_tx_queue.split();
        let (rx_prod, rx_cons) = ble_rx_queue.split();

        let mut ble_ll = LinkLayer::<BluetoothConfig>::new(device_address, ble_timer);

        let ble_r = Responder::<BluetoothConfig>::new(
            tx_prod,
            rx_cons,
            L2CAPState::new(BleChannelMap::with_attributes(BatteryServiceAttrs::new())),
        );

        let next_update = ble_ll
            .start_advertise(
                rubble::time::Duration::from_millis(200),
                &[AdStructure::CompleteLocalName("pinetime-rs")],
                &mut ble_radio,
                tx_cons,
                rx_prod,
            )
            .unwrap();
        ble_ll.timer().configure_interrupt(next_update);

        Bluetooth {
            linklayer: ble_ll,
            radio: ble_radio,
            responder: ble_r,
        }
    }

    fn handle_cmd(&mut self, cmd: Cmd) {
        self.radio.configure_receiver(cmd.radio);

        self.linklayer.timer().configure_interrupt(cmd.next_update);

        if cmd.queued_work {
            crate::tasks::ble_worker::spawn().unwrap();
        }
    }

    // Called on RADIO interrupt using ble_radio task
    pub fn on_radio(&mut self) {
        if let Some(cmd) = self.radio.recv_interrupt(
            self.linklayer.timer().now(),
            &mut self.linklayer
        ) {
            self.handle_cmd(cmd)
        }
    }

    // Called on TIMER interrupt using ble_timer task
    pub fn on_timer(&mut self) {
        let timer = self.linklayer.timer();
        if !timer.is_interrupt_pending() {
            return;
        }
        timer.clear_interrupt();

        let cmd = self.linklayer.update_timer(&mut self.radio);
        self.handle_cmd(cmd);
    }

    // Called by ble_worker task
    pub fn work(&mut self) {
        while self.responder.has_work() {
            self.responder.process_one().unwrap();
        }
    }
}
