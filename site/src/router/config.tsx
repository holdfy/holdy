import type { RouteObject } from "react-router-dom";
import { Navigate } from "react-router-dom";

import Home from "@/pages/home/page";
import NotFound from "@/pages/NotFound";
import AppLogin from "@/pages/app/AppLogin";

import BuyerLayout from "@/pages/buyer/BuyerLayout";
import AppHome from "@/pages/app/AppHome";
import AppOrders from "@/pages/app/AppOrders";
import AppOrderDetail from "@/pages/app/AppOrderDetail";
import AppWallet from "@/pages/app/AppWallet";
import AppProfile from "@/pages/app/AppProfile";
import AppPayment from "@/pages/app/AppPayment";
import AppTransactionComplete from "@/pages/app/AppTransactionComplete";

import SellerLayout from "@/pages/seller/SellerLayout";
import SellerDashboard from "@/pages/seller/SellerDashboard";
import SellerOrders from "@/pages/seller/SellerOrders";
import SellerDisputes from "@/pages/seller/SellerDisputes";
import SellerWallet from "@/pages/seller/SellerWallet";
import SellerProfile from "@/pages/seller/SellerProfile";

/**
 * Central route table — same roles as Readdy preview `router/config.tsx`,
 * extended with buyer/seller shells and legacy redirect.
 */
const routes: RouteObject[] = [
  { path: "/", element: <Home /> },
  { path: "/login", element: <AppLogin /> },
  {
    path: "/buyer",
    element: <BuyerLayout />,
    children: [
      { index: true, element: <AppHome /> },
      { path: "orders", element: <AppOrders /> },
      { path: "orders/:id", element: <AppOrderDetail /> },
      { path: "wallet", element: <AppWallet /> },
      { path: "profile", element: <AppProfile /> },
      { path: "payment", element: <AppPayment /> },
      { path: "transaction-complete", element: <AppTransactionComplete /> },
    ],
  },
  {
    path: "/seller",
    element: <SellerLayout />,
    children: [
      { index: true, element: <SellerDashboard /> },
      { path: "orders", element: <SellerOrders /> },
      { path: "orders/:id", element: <AppOrderDetail /> },
      { path: "disputes", element: <SellerDisputes /> },
      { path: "wallet", element: <SellerWallet /> },
      { path: "profile", element: <SellerProfile /> },
    ],
  },
  { path: "/app/*", element: <Navigate to="/buyer" replace /> },
  { path: "*", element: <NotFound /> },
];

export default routes;
