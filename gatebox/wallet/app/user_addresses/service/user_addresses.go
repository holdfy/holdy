package user_addressesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	user_addressesRepo "palm-pay/app/user_addresses/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type User_addressesServiceIF interface {
     GetUser_addresses(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_addressesById(ctx context.Context, id int64) (*model.User_addresses, error)
     GetUser_addressesByAddressCode(ctx context.Context, addresscode string) (*model.User_addresses, error)
     InsertUser_addresses(ctx context.Context, user_addresses *model.User_addresses) (int64, error)
     UpdateUser_addresses(ctx context.Context, user_addresses *model.User_addresses, id int64) error
     DeleteUser_addressesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     user_addressesRepo user_addressesRepo.User_addressesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewUser_addressesService(user_addressesRepo user_addressesRepo.User_addressesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         user_addressesRepo: user_addressesRepo,
		  observability:  observabilidade.NewServiceObservability("service.user_addresses"),
     }
}
func (r Resource)  GetUser_addresses(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_addressesService -> GetUser_addresses", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_addresses.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetUser_addresses.offset", offset)
	tracker.AddParam("service.GetUser_addresses.limit", limit)



	itemsPage, err = r.user_addressesRepo.GetUser_addresses(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.User_addresses); ok {
		tracker.AddResult("service.GetUser_addresses.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetUser_addressesById(ctx context.Context, id int64) (user_addresses *model.User_addresses, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_addressesService -> GetUser_addressesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_addressesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetUser_addressesById.id", id)
	user_addresses, err = r.user_addressesRepo.GetUser_addressesById(ctx, id)
	if err != nil {
		return user_addresses, errors.New(app.MsgRepositoryError)
	}

	return user_addresses, nil
}
func (r Resource)  GetUser_addressesByAddressCode(ctx context.Context, addresscode string) (user_addresses *model.User_addresses, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_addressesService -> GetUser_addressesByAddressCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetUser_addressesByAddressCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetUser_addressesByAddressCode.addresscode", addresscode)
	user_addresses, err = r.user_addressesRepo.GetUser_addressesByAddressCode(ctx, addresscode)
	if err != nil {
		return user_addresses, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetUser_addressesByAddressCode.found", user_addresses != nil)
	return user_addresses, nil
}
func (r Resource)  DeleteUser_addressesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_addressesService -> DeleteUser_addressesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteUser_addressesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteUser_addressesById.id",id)

	result, err = r.user_addressesRepo.DeleteUser_addressesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteUser_addressesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertUser_addresses(ctx context.Context,user_addresses *model.User_addresses) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_addressesService -> InsertUser_addresses", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertUser_addresses")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertUser_addresses.addresscode", user_addresses.AddressCode)
	insertedId, err = r.user_addressesRepo.InsertUser_addresses(ctx, user_addresses)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertUser_addresses.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateUser_addresses(ctx context.Context,user_addresses *model.User_addresses, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_addressesService -> UpdateUser_addresses", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateUser_addresses")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateUser_addresses.id", id)
	tracker.AddParam("service.UpdateUser_addresses.addresscode", user_addresses.AddressCode)

	err = r.user_addressesRepo.UpdateUser_addresses(ctx, user_addresses, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateUser_addresses.updated", true)

	return nil
}

