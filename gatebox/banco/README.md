# Banco Saczuck (separado do Gatebox)

Estrutura independente para simulacao de instituicao financeira externa:

- `app_banco/` -> aplicativo Flutter Banco Saczuck
- `backend_banco/` -> API bancaria dedicada em Go

## Regra de separacao
- Nao acessar DB/codigo interno do Gatebox.
- Integracao somente por API via camada `gatebox_client`.

