import type { TFunction } from "i18next";

const ORDER_DESC_KEYS: Record<string, string> = {
  "ORD-001": "mock.ord001",
  "ORD-002": "mock.ord002",
  "ORD-003": "mock.ord003",
  "ORD-004": "mock.ord004",
  "ORD-005": "mock.ord005",
  "ORD-006": "mock.ord006",
  "ORD-007": "mock.ord007",
  "ORD-008": "mock.ord008",
};

const DISPUTE_REASON_KEYS: Record<string, string> = {
  "DIS-001": "mock.dis001",
  "DIS-002": "mock.dis002",
  "DIS-003": "mock.dis003",
};

const DISPUTE_RESOLUTION_KEYS: Record<string, string> = {
  "DIS-002": "mock.dis002Resolution",
};

export function getOrderDescription(id: string, t: TFunction): string {
  const key = ORDER_DESC_KEYS[id];
  return key ? t(key) : id;
}

export function getDisputeReason(id: string, t: TFunction): string {
  const key = DISPUTE_REASON_KEYS[id];
  return key ? t(key) : id;
}

export function getDisputeResolution(id: string, t: TFunction): string | undefined {
  const key = DISPUTE_RESOLUTION_KEYS[id];
  return key ? t(key) : undefined;
}
