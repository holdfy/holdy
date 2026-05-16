package model

import (
  "time"
)
type Change_actors struct {
    ID int64 `json:"id" db:"id"`
    ActorCode string `json:"actor_code" db:"actor_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    CanAutoApprove bool `json:"can_auto_approve" db:"can_auto_approve"`
    PriorityLevel int64 `json:"priority_level" db:"priority_level"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"change_actors_category_full_count"`
}