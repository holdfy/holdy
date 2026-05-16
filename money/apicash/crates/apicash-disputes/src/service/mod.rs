//! Serviço de disputas e integrações.

mod dispute_service;

pub use dispute_service::{
    DisputeEventSink, DisputeService, NoopDisputeEventSink, PulsarDisputeEventSink,
};
