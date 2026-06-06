-- Corrige senhas de desenvolvimento quando o seed antigo usava hashes bcrypt inválidos ($2a$10$hash123).
-- Aplicar: psql -h HOST -p 5432 -U apicash -d dubai-cash -f patch-dev-auth-passwords.sql
UPDATE authentication SET password = '$2y$10$ZZGs12LC0gZ.TulY3ZS7AeTjFc/rfQhXYvWkA/sbHdMUW0XnC/qTC' WHERE username IN ('admin_gateway', 'admin_company');
UPDATE authentication SET password = '$2y$10$bxhdVlwL8.eCzhq1KV7cc.IU9AYbo/uvf.VwVi7kvucFero.0eo0S' WHERE username = 'customer1';
UPDATE authentication SET password = '$2y$10$cdvgcjsRliRoKOgWdeEeIuh3h52Bj7//tInDkSLMMlqB8iH35Ab7.' WHERE username = 'customer2';
