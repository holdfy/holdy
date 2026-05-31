import { useNavigate } from "react-router-dom";
import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardActionArea from "@mui/material/CardActionArea";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import Icon from "@mui/material/Icon";

const portals = [
  {
    label: "Cliente",
    description: "Acesso ao painel do cliente PIX",
    path: "/customer/login",
    color: "#1565c0",
    icon: "account_circle",
  },
  {
    label: "Admin",
    description: "Administração do gateway",
    path: "/admin/login",
    color: "#6a1b9a",
    icon: "admin_panel_settings",
  },
  {
    label: "Backoffice",
    description: "Operações e logs internos",
    path: "/backoffice/login",
    color: "#37474f",
    icon: "manage_accounts",
  },
];

export default function PortalIndex() {
  const navigate = useNavigate();

  return (
    <Box
      sx={{
        minHeight: "100vh",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        background: "linear-gradient(150deg, #0d1b4b 0%, #0a3d80 60%, #005b9f 100%)",
      }}
    >
      <Typography
        variant="h3"
        sx={{ color: "#fff", fontWeight: 800, letterSpacing: 2, mb: 1 }}
      >
        GateBox
      </Typography>
      <Typography
        variant="subtitle1"
        sx={{ color: "rgba(255,255,255,0.65)", mb: 6, letterSpacing: 1 }}
      >
        Selecione o tipo de acesso
      </Typography>

      <Box
        sx={{
          display: "flex",
          gap: 3,
          flexWrap: "wrap",
          justifyContent: "center",
          px: 2,
        }}
      >
        {portals.map(({ label, description, path, color, icon }) => (
          <Card
            key={path}
            sx={{
              width: 200,
              borderRadius: 3,
              boxShadow: 6,
              transition: "transform 0.15s, box-shadow 0.15s",
              "&:hover": { transform: "translateY(-6px)", boxShadow: 12 },
            }}
          >
            <CardActionArea onClick={() => navigate(path)} sx={{ py: 4, px: 2 }}>
              <CardContent sx={{ textAlign: "center", p: 0 }}>
                <Box
                  sx={{
                    width: 64,
                    height: 64,
                    borderRadius: "50%",
                    backgroundColor: color,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    mx: "auto",
                    mb: 2,
                  }}
                >
                  <Icon sx={{ color: "#fff", fontSize: "2rem !important" }}>{icon}</Icon>
                </Box>
                <Typography variant="h6" fontWeight={700}>
                  {label}
                </Typography>
                <Typography variant="body2" color="text.secondary" mt={1}>
                  {description}
                </Typography>
              </CardContent>
            </CardActionArea>
          </Card>
        ))}
      </Box>
    </Box>
  );
}
