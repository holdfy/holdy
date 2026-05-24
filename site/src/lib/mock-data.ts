export type OrderStatus = "PENDING" | "IN_CUSTODY" | "COMPLETED" | "CANCELLED";
export type PaymentStatus = "PENDING" | "IN_CUSTODY" | "RELEASED" | "REFUNDED";
export type DisputeStatus = "OPEN" | "IN_REVIEW" | "RESOLVED" | "CLOSED";

export interface Order {
  id: string;
  buyer: string;
  seller: string;
  amount: number;
  status: OrderStatus;
  date: string;
  description: string;
}

export interface Payment {
  id: string;
  orderId: string;
  amount: number;
  status: PaymentStatus;
  pixKey: string;
  date: string;
}

export interface Dispute {
  id: string;
  orderId: string;
  buyer: string;
  seller: string;
  reason: string;
  status: DisputeStatus;
  date: string;
  resolution?: string;
}

export const mockOrders: Order[] = [
  { id: "ORD-001", buyer: "Carlos Silva", seller: "TechStore", amount: 2499.90, status: "IN_CUSTODY", date: "2026-04-07", description: "iPhone 15 Pro Max" },
  { id: "ORD-002", buyer: "Ana Souza", seller: "ModaFit", amount: 349.90, status: "COMPLETED", date: "2026-04-06", description: "Nike Air Max sneakers" },
  { id: "ORD-003", buyer: "João Lima", seller: "GamerZone", amount: 4999.00, status: "PENDING", date: "2026-04-08", description: "PS5 + 2 controllers" },
  { id: "ORD-004", buyer: "Maria Santos", seller: "CasaDecor", amount: 1200.00, status: "IN_CUSTODY", date: "2026-04-05", description: "3-seat sofa" },
  { id: "ORD-005", buyer: "Pedro Costa", seller: "AutoParts", amount: 850.00, status: "CANCELLED", date: "2026-04-04", description: "Suspension kit" },
  { id: "ORD-006", buyer: "Lucas Ferreira", seller: "TechStore", amount: 3200.00, status: "COMPLETED", date: "2026-04-03", description: "MacBook Air M2" },
  { id: "ORD-007", buyer: "Juliana Oliveira", seller: "BelezaPura", amount: 189.90, status: "IN_CUSTODY", date: "2026-04-07", description: "Skincare set" },
  { id: "ORD-008", buyer: "Rafael Mendes", seller: "GamerZone", amount: 1599.00, status: "PENDING", date: "2026-04-08", description: '27" 4K monitor' },
];

export const mockPayments: Payment[] = [
  { id: "PAY-001", orderId: "ORD-001", amount: 2499.90, status: "IN_CUSTODY", pixKey: "carlos@email.com", date: "2026-04-07" },
  { id: "PAY-002", orderId: "ORD-002", amount: 349.90, status: "RELEASED", pixKey: "ana@email.com", date: "2026-04-06" },
  { id: "PAY-003", orderId: "ORD-003", amount: 4999.00, status: "PENDING", pixKey: "joao@email.com", date: "2026-04-08" },
  { id: "PAY-004", orderId: "ORD-004", amount: 1200.00, status: "IN_CUSTODY", pixKey: "maria@email.com", date: "2026-04-05" },
  { id: "PAY-005", orderId: "ORD-005", amount: 850.00, status: "REFUNDED", pixKey: "pedro@email.com", date: "2026-04-04" },
  { id: "PAY-006", orderId: "ORD-006", amount: 3200.00, status: "RELEASED", pixKey: "lucas@email.com", date: "2026-04-03" },
];

export const mockDisputes: Dispute[] = [
  { id: "DIS-001", orderId: "ORD-004", buyer: "Maria Santos", seller: "CasaDecor", reason: "Product different from listing", status: "OPEN", date: "2026-04-06" },
  { id: "DIS-002", orderId: "ORD-005", buyer: "Pedro Costa", seller: "AutoParts", reason: "Product not shipped on time", status: "RESOLVED", date: "2026-04-04", resolution: "Full refund processed" },
  { id: "DIS-003", orderId: "ORD-001", buyer: "Carlos Silva", seller: "TechStore", reason: "Box damaged on delivery", status: "IN_REVIEW", date: "2026-04-07" },
];

export const mockMetrics = {
  totalCustody: 3889.80,
  activeOrders: 5,
  openDisputes: 2,
  transactionsChart: [
    { date: "Apr 1", value: 1200 },
    { date: "Apr 2", value: 3400 },
    { date: "Apr 3", value: 3200 },
    { date: "Apr 4", value: 850 },
    { date: "Apr 5", value: 1200 },
    { date: "Apr 6", value: 349.9 },
    { date: "Apr 7", value: 2689.8 },
    { date: "Apr 8", value: 6598 },
  ],
};
