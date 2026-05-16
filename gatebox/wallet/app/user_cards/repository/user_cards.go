package user_cardsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type User_cardsRepositoryIF interface {
     GetUser_cards(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_cardsById(ctx context.Context, id int64) (*model.User_cards, error)
     GetUser_cardsByCardCode(ctx context.Context, cardcode string) (*model.User_cards, error)
     InsertUser_cards(ctx context.Context, user_cards *model.User_cards) (int64, error)
     UpdateUser_cards(ctx context.Context, user_cards *model.User_cards, id int64) error
     DeleteUser_cardsById(ctx context.Context, id int64) (bool, error)
}
 type User_cardsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewUser_cardsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *User_cardsRepository{
    return &User_cardsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("User_cards"),
     }
}
func (t User_cardsRepository)  GetUser_cards(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_cardsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_cards")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_cards.offset", offset)
	tracker.AddParam("repository.GetUser_cards.limit", limit)
	itemsPage 			= model.ItemsPage{}
	user_cardss := []model.User_cards{}

	rows, err := t.PGRead.Query(ctx, SQL_USER_CARDS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "User_cardsRepository.repository.GetUser_cardss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var user_cards model.User_cards
		err := rows.Scan(
			&user_cards.ID,
			&user_cards.CardCode,
			&user_cards.UserId,
			&user_cards.WalletId,
			&user_cards.CardToken,
			&user_cards.IdCardBrand,
			&user_cards.MaskedNumber,
			&user_cards.HolderName,
			&user_cards.ExpiryMonth,
			&user_cards.ExpiryYear,
			&user_cards.IdCardType,
			&user_cards.IdAcquirer,
			&user_cards.IsPrimary,
			&user_cards.IsActive,
			&user_cards.LastUsed,
			&user_cards.CreatedAt,
			&user_cards.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "User_cardsRepository.repository.GetUser_cardss.Scan: ", err.Error())
			return itemsPage, err
		}
		user_cardss = append(user_cardss, user_cards)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "User_cardsRepository.repository.GetUser_cardss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(user_cardss) > 0 {
		qtyRecords = user_cardss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = user_cardss

	tracker.AddResult("repository.GetUser_cards.rows_returned", len(user_cardss))
	tracker.AddResult("repository.GetUser_cards.total_count", len(user_cardss))

	return itemsPage, nil
}
func (t User_cardsRepository)  GetUser_cardsById(ctx context.Context, id int64) (user_cards *model.User_cards, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_cardsRepository -> GetUser_cardsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_cardsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_cardsById.id", id)

	user_cards = new(model.User_cards)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_CARDS_BY_ID, id)
		err = row.Scan(
			&user_cards.ID,
			&user_cards.CardCode,
			&user_cards.UserId,
			&user_cards.WalletId,
			&user_cards.CardToken,
			&user_cards.IdCardBrand,
			&user_cards.MaskedNumber,
			&user_cards.HolderName,
			&user_cards.ExpiryMonth,
			&user_cards.ExpiryYear,
			&user_cards.IdCardType,
			&user_cards.IdAcquirer,
			&user_cards.IsPrimary,
			&user_cards.IsActive,
			&user_cards.LastUsed,
			&user_cards.CreatedAt,
			&user_cards.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_cardsRepository.repository.GetUser_cardsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetUser_cardsById.found", true)
	return user_cards, nil
}
func (t User_cardsRepository)  GetUser_cardsByCardCode(ctx context.Context, cardcode string) (user_cards *model.User_cards, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_cardsRepository -> GetUser_cardsByCardCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_cardsByCardCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_cardsByCardCode.cardcode", cardcode)

	user_cards = new(model.User_cards)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_CARDS_BY_CARD_CODE, cardcode)
		err = row.Scan(
			&user_cards.ID,
			&user_cards.CardCode,
			&user_cards.UserId,
			&user_cards.WalletId,
			&user_cards.CardToken,
			&user_cards.IdCardBrand,
			&user_cards.MaskedNumber,
			&user_cards.HolderName,
			&user_cards.ExpiryMonth,
			&user_cards.ExpiryYear,
			&user_cards.IdCardType,
			&user_cards.IdAcquirer,
			&user_cards.IsPrimary,
			&user_cards.IsActive,
			&user_cards.LastUsed,
			&user_cards.CreatedAt,
			&user_cards.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_cardsRepository.repository.GetUser_cardsBycardcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return user_cards, nil
}
func (t User_cardsRepository)  DeleteUser_cardsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_cardsRepository -> DeleteUser_cardsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteUser_cardsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_USER_CARDS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"User_cardsRepository.repository.DeleteUser_cardsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteUser_cardsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteUser_cardsById.deleted", result)
	return true, err
}
func (t User_cardsRepository)  InsertUser_cards(ctx context.Context,user_cards *model.User_cards) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_cardsRepository -> InsertUser_cards", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertUser_cards")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertUser_cards.cardcode", user_cards.CardCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_USER_CARDS_INSERT,
			user_cards.CardCode,
			user_cards.UserId,
			user_cards.WalletId,
			user_cards.CardToken,
			user_cards.IdCardBrand,
			user_cards.MaskedNumber,
			user_cards.HolderName,
			user_cards.ExpiryMonth,
			user_cards.ExpiryYear,
			user_cards.IdCardType,
			user_cards.IdAcquirer,
			user_cards.IsPrimary,
			user_cards.IsActive,
			user_cards.LastUsed,
			user_cards.CreatedAt,
			user_cards.UpdatedAt,
	).Scan(&user_cards.ID)

	if err != nil {
		t.log.Error(ctx, "User_cardsRepository.repository.InsertUser_cards.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertUser_cards.inserted_id", user_cards.ID)
   return user_cards.ID, nil

}
func (t User_cardsRepository)  UpdateUser_cards(ctx context.Context,user_cards *model.User_cards, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_cardsRepository -> UpdateUser_cards", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateUser_cards")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateUser_cards.id", id)
	tracker.AddParam("repository.UpdateUser_cards.cardcode", user_cards.CardCode)

	user_cards.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_USER_CARDS_UPDATE, 
			user_cards.CardCode,
			user_cards.UserId,
			user_cards.WalletId,
			user_cards.CardToken,
			user_cards.IdCardBrand,
			user_cards.MaskedNumber,
			user_cards.HolderName,
			user_cards.ExpiryMonth,
			user_cards.ExpiryYear,
			user_cards.IdCardType,
			user_cards.IdAcquirer,
			user_cards.IsPrimary,
			user_cards.IsActive,
			user_cards.LastUsed,
			user_cards.CreatedAt,
			user_cards.UpdatedAt,
			user_cards.ID,
   )
	if err != nil {
		t.log.Error(ctx, "User_cardsRepository.repository.UpdateUser_cards.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateUser_cards.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateUser_cards.rows_affected", rowsAffected)
	return nil
}

