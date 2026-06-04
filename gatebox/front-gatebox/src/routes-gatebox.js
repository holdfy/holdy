/**
 * Rotas Gatebox - por perfil (customer, admin, backoffice)
 */

import Icon from "@mui/material/Icon";

// Layouts
import LoginCustomer from "layouts/authentication/LoginCustomer";
import LoginAdmin from "layouts/authentication/LoginAdmin";
import LoginBackoffice from "layouts/authentication/LoginBackoffice";
import CustomerDashboard from "layouts/customer/Dashboard";
import CustomerBalance from "layouts/customer/Balance";
import CustomerExtract from "layouts/customer/Extract";
import CustomerKeys from "layouts/customer/Keys";
import CustomerPixSend from "layouts/customer/PixSend";
import CustomerPixQrcode from "layouts/customer/PixQrcode";
import CustomerPixDecodeBrcode from "layouts/customer/PixDecodeBrcode";
import CustomerPixReversal from "layouts/customer/PixReversal";
import CustomerPixTransactions from "layouts/customer/PixTransactions";
import CustomerPixInvoiceStatus from "layouts/customer/PixInvoiceStatus";
import CustomerP2PSend from "layouts/customer/P2PSend";
import CustomerP2PHistory from "layouts/customer/P2PHistory";
import CustomerLimits from "layouts/customer/Limits";
import AdminDashboard from "layouts/admin/Dashboard";
import AdminCustomers from "layouts/admin/Customers";
import AdminCustomerDetail from "layouts/admin/CustomerDetail";
import AdminPixTransactions from "layouts/admin/PixTransactions";
import AdminPixTransactionDetail from "layouts/admin/PixTransactionDetail";
import AdminPartners from "layouts/admin/Partners";
import AdminWebhooks from "layouts/admin/Webhooks";
import AdminSecMed from "layouts/admin/SecMed";
import AdminSecMedDetail from "layouts/admin/SecMedDetail";
import AdminPixQrcode from "layouts/admin/PixQrcode";
import AdminKeyPix from "layouts/admin/KeyPix";
import ReportTransactions from "layouts/admin/Reports/Transactions";
import ReportInvoices from "layouts/admin/Reports/Invoices";
import ReportUsersAccounts from "layouts/admin/Reports/UsersAccounts";
import ReportMed from "layouts/admin/Reports/Med";
import ReportPartners from "layouts/admin/Reports/Partners";
import ReportWebhooks from "layouts/admin/Reports/Webhooks";
import ReportLogs from "layouts/admin/Reports/Logs";
import ReportBalances from "layouts/admin/Reports/Balances";
import ReportReversals from "layouts/admin/Reports/Reversals";
import ReportReconciliation from "layouts/admin/Reports/Reconciliation";
import ReportFees from "layouts/admin/Reports/Fees";
import ReportProfit from "layouts/admin/Reports/Profit";
import ReportExtract from "layouts/admin/Reports/Extract";
import ReportActivities from "layouts/admin/Reports/Activities";
import ReportBalanceDifferences from "layouts/admin/Reports/BalanceDifferences";
import ReportFeeReconciliation from "layouts/admin/Reports/FeeReconciliation";
import CustomerWebhooks from "layouts/customer/Webhooks";
import AdminAccount from "layouts/admin/Account";
import AdminChangePassword from "layouts/admin/ChangePassword";
import AdminDisputes from "layouts/admin/Disputes";
import AdminSettings from "layouts/admin/Settings";
import AdminHoldfyTransactions from "layouts/admin/HoldfyTransactions";
import BackofficeLogs from "layouts/backoffice/Logs";
import BackofficeAccounts from "layouts/backoffice/Accounts";
import LogoutRedirect from "components/LogoutRedirect";

// Rotas públicas (sem auth)
export const publicRoutes = [
  { path: "/customer/login", element: <LoginCustomer /> },
  { path: "/admin/login", element: <LoginAdmin /> },
  { path: "/backoffice/login", element: <LoginBackoffice /> },
];

// Rotas Customer (requer profile=customer)
export const customerRoutes = [
  { path: "/customer/dashboard", element: <CustomerDashboard /> },
  { path: "/customer/balance", element: <CustomerBalance /> },
  { path: "/customer/extract", element: <CustomerExtract /> },
  { path: "/customer/keys", element: <CustomerKeys /> },
  { path: "/customer/pix/send", element: <CustomerPixSend /> },
  { path: "/customer/pix/qrcode", element: <CustomerPixQrcode /> },
  { path: "/customer/pix/decode-brcode", element: <CustomerPixDecodeBrcode /> },
  { path: "/customer/pix/reversal", element: <CustomerPixReversal /> },
  { path: "/customer/pix/transactions", element: <CustomerPixTransactions /> },
  { path: "/customer/pix/invoice-status", element: <CustomerPixInvoiceStatus /> },
  { path: "/customer/p2p/send", element: <CustomerP2PSend /> },
  { path: "/customer/p2p/history", element: <CustomerP2PHistory /> },
  { path: "/customer/limits", element: <CustomerLimits /> },
  { path: "/customer/webhooks", element: <CustomerWebhooks /> },
  { path: "/customer/logout", element: <LogoutRedirect loginPath="/customer/login" /> },
];

// Rotas Admin (requer profile=admin)
export const adminRoutes = [
  { path: "/admin/dashboard", element: <AdminDashboard /> },
  { path: "/admin/customers", element: <AdminCustomers /> },
  { path: "/admin/customers/:id", element: <AdminCustomerDetail /> },
  { path: "/admin/pix/transactions", element: <AdminPixTransactions /> },
  { path: "/admin/holdfy/transactions", element: <AdminHoldfyTransactions /> },
  { path: "/admin/pix/transactions/:id", element: <AdminPixTransactionDetail /> },
  { path: "/admin/partners", element: <AdminPartners /> },
  { path: "/admin/webhooks", element: <AdminWebhooks /> },
  { path: "/admin/disputes", element: <AdminDisputes /> },
  { path: "/admin/sec-med", element: <AdminSecMed /> },
  { path: "/admin/sec-med/:id", element: <AdminSecMedDetail /> },
  { path: "/admin/pix/qrcode", element: <AdminPixQrcode /> },
  { path: "/admin/key-pix", element: <AdminKeyPix /> },
  { path: "/admin/account", element: <AdminAccount /> },
  { path: "/admin/reports/users-accounts", element: <ReportUsersAccounts /> },
  { path: "/admin/reports/transactions", element: <ReportTransactions /> },
  { path: "/admin/reports/invoices", element: <ReportInvoices /> },
  { path: "/admin/reports/med", element: <ReportMed /> },
  { path: "/admin/reports/partners", element: <ReportPartners /> },
  { path: "/admin/reports/webhooks", element: <ReportWebhooks /> },
  { path: "/admin/reports/logs", element: <ReportLogs /> },
  { path: "/admin/reports/balances", element: <ReportBalances /> },
  { path: "/admin/reports/reversals", element: <ReportReversals /> },
  { path: "/admin/reports/reconciliation", element: <ReportReconciliation /> },
  { path: "/admin/reports/fees", element: <ReportFees /> },
  { path: "/admin/reports/profit", element: <ReportProfit /> },
  { path: "/admin/reports/extract", element: <ReportExtract /> },
  { path: "/admin/reports/activities", element: <ReportActivities /> },
  { path: "/admin/reports/balance-differences", element: <ReportBalanceDifferences /> },
  { path: "/admin/reports/fee-reconciliation", element: <ReportFeeReconciliation /> },
  { path: "/admin/change-password", element: <AdminChangePassword /> },
  { path: "/admin/settings", element: <AdminSettings /> },
  { path: "/admin/logout", element: <LogoutRedirect loginPath="/admin/login" /> },
];

// Rotas Backoffice (requer profile=backoffice)
export const backofficeRoutes = [
  { path: "/backoffice/logs", element: <BackofficeLogs /> },
  { path: "/backoffice/accounts", element: <BackofficeAccounts /> },
  { path: "/backoffice/logout", element: <LogoutRedirect loginPath="/backoffice/login" /> },
];

// Sidenav para Customer (com component para Sidenav/Route)
export const customerSidenav = [
  {
    type: "collapse",
    name: "Menu",
    key: "menu",
    icon: <Icon fontSize="small">dashboard</Icon>,
    collapse: [
      { name: "Dashboard", key: "dashboard", route: "/customer/dashboard", component: <CustomerDashboard /> },
      { name: "Saldo", key: "balance", route: "/customer/balance", component: <CustomerBalance /> },
      { name: "Extrato", key: "extract", route: "/customer/extract", component: <CustomerExtract /> },
      { name: "Chaves PIX", key: "keys", route: "/customer/keys", component: <CustomerKeys /> },
      { name: "Enviar PIX", key: "pix-send", route: "/customer/pix/send", component: <CustomerPixSend /> },
      { name: "Gerar QR Code", key: "pix-qrcode", route: "/customer/pix/qrcode", component: <CustomerPixQrcode /> },
      { name: "Decodificar BR Code", key: "pix-decode", route: "/customer/pix/decode-brcode", component: <CustomerPixDecodeBrcode /> },
      { name: "Devolução PIX", key: "pix-reversal", route: "/customer/pix/reversal", component: <CustomerPixReversal /> },
      { name: "Transações PIX", key: "pix-tx", route: "/customer/pix/transactions", component: <CustomerPixTransactions /> },
      { name: "Status Invoice", key: "pix-invoice-status", route: "/customer/pix/invoice-status", component: <CustomerPixInvoiceStatus /> },
      { name: "P2P Enviar", key: "p2p-send", route: "/customer/p2p/send", component: <CustomerP2PSend /> },
      { name: "P2P Histórico", key: "p2p-history", route: "/customer/p2p/history", component: <CustomerP2PHistory /> },
      { name: "Limites", key: "limits", route: "/customer/limits", component: <CustomerLimits /> },
      { name: "Webhooks", key: "webhooks", route: "/customer/webhooks", component: <CustomerWebhooks /> },
      { name: "Sair", key: "logout", route: "/customer/logout", component: <LogoutRedirect loginPath="/customer/login" /> },
    ],
  },
];

// Sidenav para Admin
export const adminSidenav = [
  {
    type: "collapse",
    name: "Menu",
    key: "menu",
    icon: <Icon fontSize="small">admin_panel_settings</Icon>,
    collapse: [
      { name: "Dashboard", key: "dashboard", route: "/admin/dashboard", component: <AdminDashboard /> },
      { name: "Clientes", key: "customers", route: "/admin/customers", component: <AdminCustomers /> },
      { name: "Transações PIX", key: "pix", route: "/admin/pix/transactions", component: <AdminPixTransactions /> },
      { name: "Transações HoldFy", key: "holdfy-tx", route: "/admin/holdfy/transactions", component: <AdminHoldfyTransactions /> },
      { name: "Parceiros", key: "partners", route: "/admin/partners", component: <AdminPartners /> },
      { name: "Webhooks", key: "webhooks", route: "/admin/webhooks", component: <AdminWebhooks /> },
      { name: "Disputas", key: "disputes", route: "/admin/disputes", component: <AdminDisputes /> },
      { name: "MED", key: "sec-med", route: "/admin/sec-med", component: <AdminSecMed /> },
      { name: "QR Code PIX", key: "pix-qrcode", route: "/admin/pix/qrcode", component: <AdminPixQrcode /> },
      { name: "Chaves PIX", key: "key-pix", route: "/admin/key-pix", component: <AdminKeyPix /> },
      { name: "Conta Admin", key: "account", route: "/admin/account", component: <AdminAccount /> },
      { name: "Relatório Usuários", key: "report-users", route: "/admin/reports/users-accounts", component: <ReportUsersAccounts /> },
      { name: "Relatório Transações", key: "report-tx", route: "/admin/reports/transactions", component: <ReportTransactions /> },
      { name: "Relatório Invoices", key: "report-invoices", route: "/admin/reports/invoices", component: <ReportInvoices /> },
      { name: "Relatório MED", key: "report-med", route: "/admin/reports/med", component: <ReportMed /> },
      { name: "Relatório Parceiros", key: "report-partners", route: "/admin/reports/partners", component: <ReportPartners /> },
      { name: "Relatório Webhooks", key: "report-webhooks", route: "/admin/reports/webhooks", component: <ReportWebhooks /> },
      { name: "Relatório Logs", key: "report-logs", route: "/admin/reports/logs", component: <ReportLogs /> },
      { name: "Relatório Saldos", key: "report-balances", route: "/admin/reports/balances", component: <ReportBalances /> },
      { name: "Relatório Reversões", key: "report-reversals", route: "/admin/reports/reversals", component: <ReportReversals /> },
      { name: "Conciliação", key: "report-reconciliation", route: "/admin/reports/reconciliation", component: <ReportReconciliation /> },
      { name: "Relatório Taxas", key: "report-fees", route: "/admin/reports/fees", component: <ReportFees /> },
      { name: "Relatório Lucro", key: "report-profit", route: "/admin/reports/profit", component: <ReportProfit /> },
      { name: "Extrato Consolidado", key: "report-extract", route: "/admin/reports/extract", component: <ReportExtract /> },
      { name: "Relatório Atividades", key: "report-activities", route: "/admin/reports/activities", component: <ReportActivities /> },
      { name: "Diferenças Saldo", key: "report-balance-diff", route: "/admin/reports/balance-differences", component: <ReportBalanceDifferences /> },
      { name: "Conciliação Taxas", key: "report-fee-recon", route: "/admin/reports/fee-reconciliation", component: <ReportFeeReconciliation /> },
      { name: "Configurações", key: "settings", route: "/admin/settings", component: <AdminSettings /> },
      { name: "Trocar senha", key: "change-password", route: "/admin/change-password", component: <AdminChangePassword /> },
      { name: "Sair", key: "logout", route: "/admin/logout", component: <LogoutRedirect loginPath="/admin/login" /> },
    ],
  },
];

// Sidenav para Backoffice
export const backofficeSidenav = [
  {
    type: "collapse",
    name: "Menu",
    key: "menu",
    icon: <Icon fontSize="small">settings</Icon>,
    collapse: [
      { name: "Logs", key: "logs", route: "/backoffice/logs", component: <BackofficeLogs /> },
      { name: "Contas", key: "accounts", route: "/backoffice/accounts", component: <BackofficeAccounts /> },
      { name: "Sair", key: "logout", route: "/backoffice/logout", component: <LogoutRedirect loginPath="/backoffice/login" /> },
    ],
  },
];
