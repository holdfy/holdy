package wallet_providersRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Wallet_providersRepositoryIF interface {
     GetWallet_providers(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetWallet_providersById(ctx context.Context, id int64) (*model.Wallet_providers, error)
     GetWallet_providersByProviderCode(ctx context.Context, providercode string) (*model.Wallet_providers, error)
     InsertWallet_providers(ctx context.Context, wallet_providers *model.Wallet_providers) (int64, error)
     UpdateWallet_providers(ctx context.Context, wallet_providers *model.Wallet_providers, id int64) error
     DeleteWallet_providersById(ctx context.Context, id int64) (bool, error)
}
 type Wallet_providersRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewWallet_providersRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Wallet_providersRepository{
    return &Wallet_providersRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Wallet_providers"),
     }
}
func (t Wallet_providersRepository)  GetWallet_providers(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_providersRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_providers")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_providers.offset", offset)
	tracker.AddParam("repository.GetWallet_providers.limit", limit)
	itemsPage 			= model.ItemsPage{}
	wallet_providerss := []model.Wallet_providers{}

	rows, err := t.PGRead.Query(ctx, SQL_WALLET_PROVIDERS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Wallet_providersRepository.repository.GetWallet_providerss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var wallet_providers model.Wallet_providers
		err := rows.Scan(
			&wallet_providers.ID,
			&wallet_providers.ProviderCode,
			&wallet_providers.Name,
			&wallet_providers.Description,
			&wallet_providers.ApiEndpoint,
			&wallet_providers.RequiresToken,
			&wallet_providers.TokenDurationHours,
			&wallet_providers.IsActive,
			&wallet_providers.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Wallet_providersRepository.repository.GetWallet_providerss.Scan: ", err.Error())
			return itemsPage, err
		}
		wallet_providerss = append(wallet_providerss, wallet_providers)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Wallet_providersRepository.repository.GetWallet_providerss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(wallet_providerss) > 0 {
		qtyRecords = wallet_providerss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = wallet_providerss

	tracker.AddResult("repository.GetWallet_providers.rows_returned", len(wallet_providerss))
	tracker.AddResult("repository.GetWallet_providers.total_count", len(wallet_providerss))

	return itemsPage, nil
}
func (t Wallet_providersRepository)  GetWallet_providersById(ctx context.Context, id int64) (wallet_providers *model.Wallet_providers, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_providersRepository -> GetWallet_providersById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_providersById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_providersById.id", id)

	wallet_providers = new(model.Wallet_providers)
	row := t.PGRead.QueryRow(ctx, SQL_GET_WALLET_PROVIDERS_BY_ID, id)
		err = row.Scan(
			&wallet_providers.ID,
			&wallet_providers.ProviderCode,
			&wallet_providers.Name,
			&wallet_providers.Description,
			&wallet_providers.ApiEndpoint,
			&wallet_providers.RequiresToken,
			&wallet_providers.TokenDurationHours,
			&wallet_providers.IsActive,
			&wallet_providers.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Wallet_providersRepository.repository.GetWallet_providersById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetWallet_providersById.found", true)
	return wallet_providers, nil
}
func (t Wallet_providersRepository)  GetWallet_providersByProviderCode(ctx context.Context, providercode string) (wallet_providers *model.Wallet_providers, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_providersRepository -> GetWallet_providersByProviderCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_providersByProviderCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_providersByProviderCode.providercode", providercode)

	wallet_providers = new(model.Wallet_providers)
	row := t.PGRead.QueryRow(ctx, SQL_GET_WALLET_PROVIDERS_BY_PROVIDER_CODE, providercode)
		err = row.Scan(
			&wallet_providers.ID,
			&wallet_providers.ProviderCode,
			&wallet_providers.Name,
			&wallet_providers.Description,
			&wallet_providers.ApiEndpoint,
			&wallet_providers.RequiresToken,
			&wallet_providers.TokenDurationHours,
			&wallet_providers.IsActive,
			&wallet_providers.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Wallet_providersRepository.repository.GetWallet_providersByprovidercode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return wallet_providers, nil
}
func (t Wallet_providersRepository)  DeleteWallet_providersById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_providersRepository -> DeleteWallet_providersById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteWallet_providersById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_WALLET_PROVIDERS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Wallet_providersRepository.repository.DeleteWallet_providersById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteWallet_providersById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteWallet_providersById.deleted", result)
	return true, err
}
func (t Wallet_providersRepository)  InsertWallet_providers(ctx context.Context,wallet_providers *model.Wallet_providers) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_providersRepository -> InsertWallet_providers", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertWallet_providers")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertWallet_providers.providercode", wallet_providers.ProviderCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_WALLET_PROVIDERS_INSERT,
			wallet_providers.ProviderCode,
			wallet_providers.Name,
			wallet_providers.Description,
			wallet_providers.ApiEndpoint,
			wallet_providers.RequiresToken,
			wallet_providers.TokenDurationHours,
			wallet_providers.IsActive,
			wallet_providers.CreatedAt,
	).Scan(&wallet_providers.ID)

	if err != nil {
		t.log.Error(ctx, "Wallet_providersRepository.repository.InsertWallet_providers.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertWallet_providers.inserted_id", wallet_providers.ID)
   return wallet_providers.ID, nil

}
func (t Wallet_providersRepository)  UpdateWallet_providers(ctx context.Context,wallet_providers *model.Wallet_providers, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_providersRepository -> UpdateWallet_providers", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateWallet_providers")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateWallet_providers.id", id)
	tracker.AddParam("repository.UpdateWallet_providers.providercode", wallet_providers.ProviderCode)

	wallet_providers.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_WALLET_PROVIDERS_UPDATE, 
			wallet_providers.ProviderCode,
			wallet_providers.Name,
			wallet_providers.Description,
			wallet_providers.ApiEndpoint,
			wallet_providers.RequiresToken,
			wallet_providers.TokenDurationHours,
			wallet_providers.IsActive,
			wallet_providers.CreatedAt,
			wallet_providers.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Wallet_providersRepository.repository.UpdateWallet_providers.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateWallet_providers.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateWallet_providers.rows_affected", rowsAffected)
	return nil
}

