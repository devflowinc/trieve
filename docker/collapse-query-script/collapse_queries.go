package main

import (
	"context"
	"database/sql"
	"fmt"
	"log"
	"math"
	"os"
	"runtime"
	"strings"
	"sync"
	"time"

	"github.com/ClickHouse/clickhouse-go/v2"
	"github.com/google/uuid"
	"github.com/joho/godotenv"
)

type SearchQuery struct {
	ID          uuid.UUID
	Query       string
	TopScore    float64
	CreatedAt   time.Time
	SearchType  string
	ReqParams   string
	Latency     float64
	Results     []string
	QueryVector []float32
	IsDuplicate int
	QueryRating string
	DatasetID   uuid.UUID
}

type DatasetLastCollapsed struct {
	DatasetID     uuid.UUID
	LastCollapsed time.Time
}

func main() {
	if err := godotenv.Load(); err != nil {
		log.Printf("Warning: Error loading .env file: %v", err)
	}

	conn, err := initClickHouseConnection()
	if err != nil {
		log.Fatalf("Failed to connect to ClickHouse: %v", err)
	}
	defer conn.Close()

	datasets, err := getDatasets(conn)
	if err != nil {
		log.Fatalf("Failed to get datasets: %v", err)
	}

	var wg sync.WaitGroup
	maxWorkers := runtime.NumCPU() * 2 // Use 2x CPU cores for IO-bound work
	semaphore := make(chan struct{}, maxWorkers)

	for _, datasetID := range datasets {
		semaphore <- struct{}{} // Acquire semaphore
		wg.Add(1)
		go func(dsID uuid.UUID) {
			defer wg.Done()
			defer func() { <-semaphore }() // Release semaphore
			processDataset(conn, dsID)
		}(datasetID)
	}
	wg.Wait()
}

func initClickHouseConnection() (*sql.DB, error) {
	dsn := os.Getenv("CLICKHOUSE_DSN")
	if dsn == "" {
		return nil, fmt.Errorf("CLICKHOUSE_DSN environment variable is not set")
	}

	opts, err := clickhouse.ParseDSN(dsn)
	if err != nil {
		return nil, fmt.Errorf("failed to parse DSN: %v", err)
	}

	opts.Compression = &clickhouse.Compression{
		Method: clickhouse.CompressionLZ4,
	}

	opts.Settings = clickhouse.Settings{
		"async_insert":          "1",
		"wait_for_async_insert": "0",
	}

	conn := clickhouse.OpenDB(opts)

	conn.SetMaxIdleConns(5)
	conn.SetMaxOpenConns(10)
	conn.SetConnMaxLifetime(time.Hour)

	return conn, nil
}

func getDatasets(conn *sql.DB) ([]uuid.UUID, error) {
	query := "SELECT DISTINCT dataset_id FROM default.search_queries"

	rows, err := conn.Query(query)
	if err != nil {
		return nil, fmt.Errorf("query error: %v", err)
	}
	defer rows.Close()

	var datasets []uuid.UUID
	for rows.Next() {
		var datasetID uuid.UUID
		if err := rows.Scan(&datasetID); err != nil {
			return nil, fmt.Errorf("scan error: %v", err)
		}
		datasets = append(datasets, datasetID)
	}

	return datasets, nil
}

func getDatasetLastCollapsed(conn *sql.DB, datasetID uuid.UUID) (time.Time, bool, error) {
	query := `
		SELECT last_collapsed
		FROM default.last_collapsed_dataset
		WHERE dataset_id = ?
	`

	var lastCollapsed time.Time
	err := conn.QueryRow(query, datasetID).Scan(&lastCollapsed)
	if err == sql.ErrNoRows {
		return time.Time{}, false, nil
	} else if err != nil {
		return time.Time{}, false, fmt.Errorf("query error: %v", err)
	}

	return lastCollapsed, true, nil
}

func setDatasetLastCollapsed(conn *sql.DB, datasetID uuid.UUID, lastCollapsed time.Time) error {
	query := `
		INSERT INTO default.last_collapsed_dataset (id, last_collapsed, dataset_id, created_at)
		VALUES (?, ?, ?, ?)
	`

	_, err := conn.Exec(query, uuid.New(), lastCollapsed, datasetID, time.Now())
	if err != nil {
		return fmt.Errorf("insert error: %v", err)
	}

	return nil
}

func getSearchQueries(conn *sql.DB, datasetID uuid.UUID, limit int, offset *time.Time) ([]SearchQuery, error) {
	var query string
	var args []any

	if offset != nil {
		query = `
			SELECT id, query, top_score, created_at, search_type, request_params, latency, results, 
			       query_vector, is_duplicate, query_rating, dataset_id
			FROM default.search_queries 
			WHERE dataset_id = ? AND created_at >= ? AND search_type != 'rag'
			ORDER BY created_at DESC
			LIMIT ?
		`
		args = []any{datasetID, *offset, limit}
	} else {
		query = `
			SELECT id, query, top_score, created_at, search_type, request_params, latency, results, 
			       query_vector, is_duplicate, query_rating, dataset_id
			FROM default.search_queries 
			WHERE dataset_id = ? AND is_duplicate = 0 AND search_type != 'rag'
			ORDER BY created_at DESC
			LIMIT ?
		`
		args = []any{datasetID, limit}
	}

	rows, err := conn.Query(query, args...)
	if err != nil {
		return nil, fmt.Errorf("query error: %v", err)
	}
	defer rows.Close()

	var results []SearchQuery
	for rows.Next() {
		var sq SearchQuery
		if err := rows.Scan(
			&sq.ID, &sq.Query, &sq.TopScore, &sq.CreatedAt, &sq.SearchType,
			&sq.ReqParams, &sq.Latency, &sq.Results, &sq.QueryVector,
			&sq.IsDuplicate, &sq.QueryRating, &sq.DatasetID,
		); err != nil {
			return nil, fmt.Errorf("scan error: %v", err)
		}
		results = append(results, sq)
	}

	return results, nil
}

func collapseQueries(queries []SearchQuery, timeWindow float64) []SearchQuery {
	if len(queries) <= 1 {
		return nil
	}

	// Sort queries by timestamp
	sortByTimestamp(queries)

	// Group queries that might be part of the same typing sequence
	var typingSequences [][]SearchQuery
	currentSequence := []SearchQuery{queries[0]}

	for i := 1; i < len(queries); i++ {
		currentQuery := queries[i]
		prevQuery := currentSequence[len(currentSequence)-1]

		timeDiff := currentQuery.CreatedAt.Sub(prevQuery.CreatedAt).Seconds()

		currentText := strings.ToLower(strings.TrimSpace(currentQuery.Query))
		prevText := strings.ToLower(strings.TrimSpace(prevQuery.Query))

		// Check if queries are related
		isRelated := isPrefix(currentText, prevText) ||
			isPrefix(prevText, currentText) ||
			calculateOverlap(currentText, prevText) > 0.8

		if timeDiff <= timeWindow && isRelated {
			currentSequence = append(currentSequence, currentQuery)
		} else {
			typingSequences = append(typingSequences, currentSequence)
			currentSequence = []SearchQuery{currentQuery}
		}
	}

	if len(currentSequence) > 0 {
		typingSequences = append(typingSequences, currentSequence)
	}

	// For each sequence, keep only the longest query and mark others as duplicates
	var duplicates []SearchQuery
	for _, sequence := range typingSequences {
		if len(sequence) > 1 {
			// Find the longest query in the sequence
			longestIdx := 0
			maxLen := len(strings.TrimSpace(sequence[0].Query))

			for i := 1; i < len(sequence); i++ {
				queryLen := len(strings.TrimSpace(sequence[i].Query))
				if queryLen > maxLen {
					maxLen = queryLen
					longestIdx = i
				}
			}

			// Mark all other queries in the sequence as duplicates
			for i := range sequence {
				if i != longestIdx {
					duplicate := sequence[i]
					duplicate.IsDuplicate = 1
					duplicates = append(duplicates, duplicate)
				}
			}
		}
	}

	return duplicates
}

func sortByTimestamp(queries []SearchQuery) {
	for i := 0; i < len(queries)-1; i++ {
		for j := i + 1; j < len(queries); j++ {
			if queries[i].CreatedAt.After(queries[j].CreatedAt) {
				queries[i], queries[j] = queries[j], queries[i]
			}
		}
	}
}

func isPrefix(s1, s2 string) bool {
	return strings.HasPrefix(s1, s2) || strings.HasPrefix(s2, s1)
}

func calculateOverlap(s1, s2 string) float64 {
	if len(s1) == 0 || len(s2) == 0 {
		return 0.0
	}

	// Create character sets
	set1 := make(map[rune]bool)
	set2 := make(map[rune]bool)

	for _, ch := range s1 {
		set1[ch] = true
	}

	for _, ch := range s2 {
		set2[ch] = true
	}

	// Calculate intersection
	var intersection int
	for ch := range set1 {
		if set2[ch] {
			intersection++
		}
	}

	// Calculate Jaccard similarity
	maxLen := math.Max(float64(len(set1)), float64(len(set2)))
	return float64(intersection) / maxLen
}

func insertDuplicateRows(conn *sql.DB, duplicates []SearchQuery) error {
	if len(duplicates) == 0 {
		return nil
	}

	// Use a transaction for bulk inserts
	tx, err := conn.Begin()
	if err != nil {
		return fmt.Errorf("transaction begin error: %v", err)
	}

	stmt, err := tx.Prepare(`
		INSERT INTO default.search_queries (
			id, query, top_score, created_at, search_type, request_params, latency, 
			results, query_vector, is_duplicate, query_rating, dataset_id
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
	`)
	if err != nil {
		tx.Rollback()
		return fmt.Errorf("prepare statement error: %v", err)
	}
	defer stmt.Close()

	// Batch insert duplicates
	batchSize := 100
	for i := 0; i < len(duplicates); i += batchSize {
		end := i + batchSize
		if end > len(duplicates) {
			end = len(duplicates)
		}

		batch := duplicates[i:end]
		errCh := make(chan error, len(batch))
		var wg sync.WaitGroup

		// Use a semaphore to limit concurrent executions
		sem := make(chan struct{}, 10)

		for _, dup := range batch {
			wg.Add(1)
			sem <- struct{}{}

			go func(d SearchQuery) {
				defer wg.Done()
				defer func() { <-sem }()

				_, err := stmt.Exec(
					d.ID, d.Query, d.TopScore, d.CreatedAt, d.SearchType,
					d.ReqParams, d.Latency, d.Results, d.QueryVector,
					d.IsDuplicate, d.QueryRating, d.DatasetID,
				)

				if err != nil {
					errCh <- fmt.Errorf("exec error: %v", err)
				}
			}(dup)
		}

		wg.Wait()
		close(errCh)

		for err := range errCh {
			tx.Rollback()
			return err
		}
	}

	if err := tx.Commit(); err != nil {
		return fmt.Errorf("commit error: %v", err)
	}

	return nil
}

func processDataset(conn *sql.DB, datasetID uuid.UUID) error {
	log.Printf("Processing dataset %s", datasetID)

	// Get last collapsed timestamp
	lastCollapsed, exists, err := getDatasetLastCollapsed(conn, datasetID)
	if err != nil {
		log.Printf("Error getting last collapsed for dataset %s: %v", datasetID, err)
		return err
	}

	log.Printf("Collapsing dataset %s from %v", datasetID, lastCollapsed)

	var offset *time.Time
	if exists {
		offset = &lastCollapsed
	}

	numDuplicates := 0
	limit := 5000
	timeWindow := 5.0

	// Get initial batch of queries
	queries, err := getSearchQueries(conn, datasetID, limit, offset)
	if err != nil {
		log.Printf("Error getting search queries for dataset %s: %v", datasetID, err)
		return err
	}

	// Process queries in batches
	for len(queries) > 0 {
		// Update last processed timestamp
		if len(queries) > 0 {
			newLastCollapsed := queries[0].CreatedAt
			lastCollapsed = newLastCollapsed
			offset = &lastCollapsed
		}

		_, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
		defer cancel()

		// Process with timeout
		duplicates := collapseQueries(queries, timeWindow)
		numDuplicates += len(duplicates)

		if len(duplicates) > 0 {
			if err := insertDuplicateRows(conn, duplicates); err != nil {
				log.Printf("Error inserting duplicates for dataset %s: %v", datasetID, err)
				cancel()
				return err
			}
		}

		// Get next batch
		newQueries, err := getSearchQueries(conn, datasetID, limit, offset)
		if err != nil {
			log.Printf("Error getting next batch for dataset %s: %v", datasetID, err)
			cancel()
			return err
		}

		// Check if we've processed all queries
		if len(newQueries) == 0 || (len(newQueries) > 0 && len(queries) > 0 && newQueries[len(newQueries)-1].ID == queries[len(queries)-1].ID) {
			break
		}

		queries = newQueries
	}

	// Update last collapsed timestamp
	if offset != nil {
		if err := setDatasetLastCollapsed(conn, datasetID, *offset); err != nil {
			log.Printf("Error setting last collapsed for dataset %s: %v", datasetID, err)
		}
	}

	log.Printf("Processed dataset %s, marked %d duplicates", datasetID, numDuplicates)
	return nil
}
