import { Card, CardContent, Typography, Box } from "@mui/material";

export default function StatCard({ title, value, subtitle, icon, color = "primary.main" }) {
  return (
    <Card elevation={2}>
      <CardContent>
        <Box display="flex" alignItems="flex-start" justifyContent="space-between">
          <Box>
            <Typography variant="body2" color="text.secondary" fontWeight={600} textTransform="uppercase" letterSpacing={1}>
              {title}
            </Typography>
            <Typography variant="h4" fontWeight={700} mt={0.5}>
              {value}
            </Typography>
            {subtitle && (
              <Typography variant="body2" color="text.secondary" mt={0.5}>
                {subtitle}
              </Typography>
            )}
          </Box>
          {icon && (
            <Box sx={{ bgcolor: color, borderRadius: 2, p: 1.5, display: "flex", color: "white" }}>
              {icon}
            </Box>
          )}
        </Box>
      </CardContent>
    </Card>
  );
}
