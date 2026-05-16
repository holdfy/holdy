package model
  type ItemsPage struct {
       Offset int64       `json:"offset"`
       Limit  int64       `json:"limit"`
       Total  int64       `json:"total"`
       Items  interface{} `json:"items"`
}