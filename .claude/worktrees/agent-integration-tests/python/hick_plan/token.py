"""Token attenuation helpers for capability restriction."""


class TokenAttenuator:
    """Builds a list of macaroon caveat strings to restrict a capability token."""

    def __init__(self) -> None:
        self._caveats: list[str] = []

    def restrict_api_sink(self, sink_type: str) -> "TokenAttenuator":
        self._caveats.append(f"api-sink = {sink_type}")
        return self

    def set_expiry(self, iso_timestamp: str) -> "TokenAttenuator":
        self._caveats.append(f"expires-at = {iso_timestamp}")
        return self

    def set_max_calls(self, sink_type: str, count: int) -> "TokenAttenuator":
        self._caveats.append(f"max-calls = {sink_type}:{count}")
        return self

    def restrict_recipient(self, sink_type: str, recipient: str) -> "TokenAttenuator":
        self._caveats.append(f"recipient = {sink_type}:{recipient}")
        return self

    def restrict_data_access(self, pattern: str) -> "TokenAttenuator":
        self._caveats.append(f"data-access = {pattern}")
        return self

    def set_purpose(self, context: str, purpose: str) -> "TokenAttenuator":
        self._caveats.append(f"purpose = {context}:{purpose}")
        return self

    def to_caveats(self) -> list[str]:
        return list(self._caveats)

    def __repr__(self) -> str:
        return f"TokenAttenuator(caveats={self._caveats!r})"
