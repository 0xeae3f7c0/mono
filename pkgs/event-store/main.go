// Command event-store is the offline/edge store for activity events: SQLite
// through GORM (pure-Go driver, no cgo), used where events-api is not reachable
// and replayed into it later.
//
//	event-store -db events.db add "o/r push"
//	event-store -db events.db list
package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/glebarez/sqlite"
	"gorm.io/gorm"
)

// Event is one recorded activity event awaiting replay.
type Event struct {
	ID       uint `gorm:"primaryKey"`
	Title    string
	Replayed bool
}

func open(path string) (*gorm.DB, error) {
	db, err := gorm.Open(sqlite.Open(path), &gorm.Config{})
	if err != nil {
		return nil, err
	}
	return db, db.AutoMigrate(&Event{})
}

func add(db *gorm.DB, title string) (Event, error) {
	event := Event{Title: title}
	return event, db.Create(&event).Error
}

func list(db *gorm.DB) ([]Event, error) {
	var events []Event
	return events, db.Order("id").Find(&events).Error
}

func main() {
	path := flag.String("db", "events.db", "SQLite database file")
	flag.Parse()
	db, err := open(*path)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	switch flag.Arg(0) {
	case "add":
		task, err := add(db, flag.Arg(1))
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
		fmt.Printf("added #%d\n", task.ID)
	case "list", "":
		tasks, err := list(db)
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
		for _, t := range tasks {
			fmt.Printf("#%d %s\n", t.ID, t.Title)
		}
	default:
		fmt.Fprintln(os.Stderr, "usage: event-store [-db file] add <title> | list")
		os.Exit(2)
	}
}
