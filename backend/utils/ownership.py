"""Per-browser ownership helpers for lightweight data isolation."""
import uuid
from flask import g, request


OWNER_COOKIE_NAME = "banana_owner_id"
OWNER_HEADER_NAME = "X-Owner-Id"


def resolve_request_owner_id() -> str:
    """Get owner id from request header/cookie, or generate one for this request."""
    owner_id = (request.headers.get(OWNER_HEADER_NAME) or "").strip()
    if not owner_id:
        owner_id = (request.cookies.get(OWNER_COOKIE_NAME) or "").strip()
    if not owner_id:
        owner_id = str(uuid.uuid4())
        g._banana_owner_id_new = owner_id
    g.banana_owner_id = owner_id
    return owner_id


def get_request_owner_id() -> str:
    """Return resolved owner id for current request."""
    owner_id = getattr(g, "banana_owner_id", None)
    if owner_id:
        return owner_id
    return resolve_request_owner_id()

