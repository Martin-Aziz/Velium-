from fastapi.testclient import TestClient

from app.main import app, build_markdown, derive_title


client = TestClient(app)


def test_derive_title_returns_human_readable_title() -> None:
    title = derive_title(
        "design an incident response runbook for platform outages"
    )
    assert title == "Design An Incident Response Runbook For Platform Outages"


def test_build_markdown_contains_expected_sections() -> None:
    markdown = build_markdown("Create a monthly performance review summary")
    assert markdown.startswith("# ")
    assert "## Executive Summary" in markdown
    assert "## Key Insights" in markdown
    assert "## Recommended Next Actions" in markdown


def test_generate_markdown_endpoint_returns_payload() -> None:
    response = client.post(
        "/internal/generate-markdown",
        json={"prompt": "Create a technical design memo for API versioning"},
    )

    assert response.status_code == 200
    body = response.json()
    assert "markdown" in body
    assert body["markdown"].startswith("# ")
