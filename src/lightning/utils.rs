use lightning_invoice::{Bolt11Invoice, SignedRawBolt11Invoice};

pub fn decode_invoice(payment_request: String) -> Bolt11Invoice {
    Bolt11Invoice::from_signed(payment_request.parse::<SignedRawBolt11Invoice>().unwrap()).unwrap()
}
