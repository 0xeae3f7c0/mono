// Command webhooks receives GitHub webhooks, verifies their HMAC signature
// (X-Hub-Signature-256) and forwards push, pull_request, workflow_run and
// release events to events-api. Exposes Prometheus metrics at /metrics.
//
//	WEBHOOK_SECRET=... EVENTS_URL=http://events-api:3000 webhooks
package main

import (
	"bytes"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"io"
	"net/http"
	"os"

	"github.com/gin-contrib/requestid"
	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"go.uber.org/zap"
)

// forwarded lists the GitHub event types sent on to events-api.
var forwarded = map[string]bool{"push": true, "pull_request": true, "workflow_run": true, "release": true}

// Event is the payload events-api accepts.
type Event struct {
	Repo  string `json:"repo"`
	Kind  string `json:"kind"`
	Actor string `json:"actor"`
}

// Forwarder sends an event onward; replaced in tests.
type Forwarder func(Event) error

// validSignature reports whether header ("sha256=<hex>") is the HMAC of body.
func validSignature(secret, header string, body []byte) bool {
	const prefix = "sha256="
	if len(header) <= len(prefix) || header[:len(prefix)] != prefix {
		return false
	}
	got, err := hex.DecodeString(header[len(prefix):])
	if err != nil {
		return false
	}
	mac := hmac.New(sha256.New, []byte(secret))
	mac.Write(body)
	return hmac.Equal(got, mac.Sum(nil))
}

func newRouter(log *zap.Logger, reg *prometheus.Registry, secret string, forward Forwarder) *gin.Engine {
	received := prometheus.NewCounterVec(
		prometheus.CounterOpts{Name: "webhooks_received_total", Help: "Webhooks by event type and outcome."},
		[]string{"event", "outcome"},
	)
	reg.MustRegister(received)

	r := gin.New()
	r.Use(gin.Recovery(), requestid.New())
	r.GET("/health", func(c *gin.Context) { c.JSON(http.StatusOK, gin.H{"ok": true}) })
	r.GET("/metrics", gin.WrapH(promhttp.HandlerFor(reg, promhttp.HandlerOpts{})))
	r.POST("/github", func(c *gin.Context) {
		event := c.GetHeader("X-GitHub-Event")
		body, err := io.ReadAll(io.LimitReader(c.Request.Body, 1<<20))
		if err != nil || !validSignature(secret, c.GetHeader("X-Hub-Signature-256"), body) {
			received.WithLabelValues(event, "rejected").Inc()
			c.JSON(http.StatusUnauthorized, gin.H{"error": "invalid signature"})
			return
		}
		if !forwarded[event] {
			received.WithLabelValues(event, "ignored").Inc()
			c.Status(http.StatusNoContent)
			return
		}
		var payload struct {
			Repository struct {
				FullName string `json:"full_name"`
			} `json:"repository"`
			Sender struct {
				Login string `json:"login"`
			} `json:"sender"`
		}
		if err := json.Unmarshal(body, &payload); err != nil || payload.Repository.FullName == "" {
			received.WithLabelValues(event, "malformed").Inc()
			c.JSON(http.StatusBadRequest, gin.H{"error": "missing repository"})
			return
		}
		if err := forward(Event{Repo: payload.Repository.FullName, Kind: event, Actor: payload.Sender.Login}); err != nil {
			log.Error("forward failed", zap.String("id", requestid.Get(c)), zap.Error(err))
			received.WithLabelValues(event, "failed").Inc()
			c.JSON(http.StatusBadGateway, gin.H{"error": "events-api unavailable"})
			return
		}
		received.WithLabelValues(event, "forwarded").Inc()
		c.Status(http.StatusAccepted)
	})
	return r
}

// httpForwarder posts events to events-api at baseURL.
func httpForwarder(baseURL string) Forwarder {
	return func(e Event) error {
		body, _ := json.Marshal(e)
		res, err := http.Post(baseURL+"/events", "application/json", bytes.NewReader(body))
		if err != nil {
			return err
		}
		defer res.Body.Close()
		if res.StatusCode >= 300 {
			return &statusError{res.StatusCode}
		}
		return nil
	}
}

type statusError struct{ code int }

func (e *statusError) Error() string { return "events-api returned " + http.StatusText(e.code) }

func main() {
	log, _ := zap.NewProduction()
	defer log.Sync()
	secret := os.Getenv("WEBHOOK_SECRET")
	if secret == "" {
		log.Fatal("WEBHOOK_SECRET is required")
	}
	events := os.Getenv("EVENTS_URL")
	if events == "" {
		events = "http://127.0.0.1:3000"
	}
	addr := os.Getenv("ADDR")
	if addr == "" {
		addr = ":8080"
	}
	log.Info("listening", zap.String("addr", addr))
	if err := newRouter(log, prometheus.NewRegistry(), secret, httpForwarder(events)).Run(addr); err != nil {
		log.Fatal("server stopped", zap.Error(err))
	}
}
