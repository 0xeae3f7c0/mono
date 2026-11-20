// webhooks: Verifies GitHub webhook signatures, forwards events to `events-api`, Prometheus metrics (commit platform, pkgs/webhooks)
package main

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"go.uber.org/zap"
)

const secret = "s3cret"

func sign(body string) string {
	mac := hmac.New(sha256.New, []byte(secret))
	mac.Write([]byte(body))
	return "sha256=" + hex.EncodeToString(mac.Sum(nil))
}

func post(r http.Handler, event, body, signature string) *httptest.ResponseRecorder {
	req := httptest.NewRequest(http.MethodPost, "/github", strings.NewReader(body))
	req.Header.Set("X-GitHub-Event", event)
	req.Header.Set("X-Hub-Signature-256", signature)
	w := httptest.NewRecorder()
	r.ServeHTTP(w, req)
	return w
}

func setup() (*gin.Engine, *[]Event) {
	gin.SetMode(gin.TestMode)
	var got []Event
	r := newRouter(zap.NewNop(), prometheus.NewRegistry(), secret, func(e Event) error {
		got = append(got, e)
		return nil
	})
	return r, &got
}

func TestForwardsSignedEvents(t *testing.T) {
	r, got := setup()
	body := `{"repository":{"full_name":"o/r"},"sender":{"login":"ada"}}`
	assert.Equal(t, http.StatusAccepted, post(r, "push", body, sign(body)).Code)
	assert.Equal(t, []Event{{Repo: "o/r", Kind: "push", Actor: "ada"}}, *got)
}

func TestRejectsBadSignatures(t *testing.T) {
	r, got := setup()
	body := `{"repository":{"full_name":"o/r"}}`
	assert.Equal(t, http.StatusUnauthorized, post(r, "push", body, sign("other")).Code)
	assert.Equal(t, http.StatusUnauthorized, post(r, "push", body, "").Code)
	assert.Empty(t, *got)
}

func TestIgnoresOtherEventsAndCountsThem(t *testing.T) {
	r, got := setup()
	body := `{"zen":"hi"}`
	assert.Equal(t, http.StatusNoContent, post(r, "ping", body, sign(body)).Code)
	assert.Empty(t, *got)

	w := httptest.NewRecorder()
	r.ServeHTTP(w, httptest.NewRequest(http.MethodGet, "/metrics", nil))
	assert.Contains(t, w.Body.String(), `webhooks_received_total{event="ping",outcome="ignored"} 1`)
}

func TestValidSignature(t *testing.T) {
	assert.True(t, validSignature(secret, sign("x"), []byte("x")))
	assert.False(t, validSignature(secret, "sha256=zz", []byte("x")))
	assert.False(t, validSignature(secret, "sha1=abc", []byte("x")))
}
