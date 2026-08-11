# Hedra API v3 CLI Reference

Full command reference for `hedra`.

## Commands

- [`hedra billing`](#hedra-billing)
- [`hedra files`](#hedra-files)
- [`hedra jobs`](#hedra-jobs)
- [`hedra keys`](#hedra-keys)
- [`hedra log-drains`](#hedra-log-drains)
- [`hedra models`](#hedra-models)
- [`hedra tokens`](#hedra-tokens)
- [`hedra webhooks`](#hedra-webhooks)

---

### `hedra billing`

#### `hedra billing get-balance`

Get Balance

`GET /balance`

#### `hedra billing get-usage`

Get Usage

`GET /usage`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--start` | `string` | No | Window start (inclusive, ISO-8601); defaults to 7 days before `end`. Bounds job-creation time. |
| `--end` | `string` | No | Window end (exclusive, ISO-8601); defaults to now. The window is capped at 90 days. |
| `--group-by` | `UsageGroupBy` | No | One summary row (`total`), one per UTC day (`day`), or one per model (`model`). |

---

### `hedra files`

#### `hedra files upload`

Store a file and return a short-lived URL to pass in a model's `input`.

Free, and available on an empty API wallet — funding is enforced when you
submit a generation, not when you upload its inputs. `GET /v3/balance`
reports what the wallet holds.

`POST /files`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

---

### `hedra jobs`

#### `hedra jobs get`

Get Job

`GET /jobs/{job_id}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--job-id` | `string` | Yes | The job's id (`job_<uuid>`). |

#### `hedra jobs get-status`

Get Job Status

`GET /jobs/{job_id}/status`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--job-id` | `string` | Yes | The job's id (`job_<uuid>`). |
| `--logs-after` | `string` | No | Tail this job's lifecycle events incrementally: returns only events newer than this cursor, plus `logs_next_cursor` to send on the next poll. Pass `start` to begin from the job's first event. Omit it and the response carries no events at all — the default polling shape is unchanged. |

#### `hedra jobs list`

List Jobs

`GET /jobs`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--limit` | `integer` | No | Maximum items per page. |
| `--cursor` | `string` | No | Opaque cursor from the previous page's `next_cursor`; omit for the first page. |

#### `hedra jobs list-job-logs`

List Job Logs

`GET /jobs/{job_id}/logs`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--job-id` | `string` | Yes | The job's id (`job_<uuid>`). |
| `--limit` | `integer` | No | Maximum items per page. |
| `--cursor` | `string` | No | Opaque cursor from the previous page's `next_cursor`; omit for the first page. |

#### `hedra jobs stream`

Stream Job

`GET /jobs/{job_id}/stream`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--job-id` | `string` | Yes | The job's id (`job_<uuid>`). |
| `--last-event-id` | `string` | No | Resume after this event id — the standard SSE reconnect header; browsers' EventSource sends it automatically. |

#### `hedra jobs submit`

Runs any model in the catalog by its public id, with `input` passed through untyped — the same call the typed operations below make, minus the compile-time schema.

Reach for it when the model is not known ahead of time: a client generated before a model shipped can still run it, and an id read from `GET /v3/models` at runtime needs no regeneration. Prefer the typed operation whenever your client already has one — `input` here is validated against the same published schema (`GET /v3/models/{model}`), so a bad field is a `400` at submit rather than an error before the call.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/{model}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes | The model's public id (`GET /v3/models`). |
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-dreamina-31`

Ultra high quality generations for professional grade images.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/dreamina-31`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-elevenlabs-flash-multilingual-v2`

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/elevenlabs-flash-multilingual-v2`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-elevenlabs-flash-v2`

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/elevenlabs-flash-v2`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-elevenlabs-multilingual-v2`

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/elevenlabs-multilingual-v2`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-elevenlabs-v3`

ElevenLabs V3

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/elevenlabs-v3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-flux-11-pro`

Premium color depth and clarity when you want campaign-ready art that feels handcrafted.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/flux-11-pro`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-flux-11-ultra`

The big-canvas choice for ultra-high-res images and flagship visuals.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/flux-11-ultra`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-flux-3`

Black Forest Labs FLUX.3 text-to-video with native audio.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/flux-3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-flux-dev`

Fast and light for quick concepts or high-volume social posts on a budget.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/flux-dev`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-flux-kontext-max`

Highest-fidelity reference-image support for complex, multi-element scenes and perfectly matched branded visuals.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/flux-kontext-max`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-flux-kontext-pro`

Reference-image support for character, brand, or style consistency.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/flux-kontext-pro`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-flux2-flex`

Image creation and editing with FLUX.2 [flex] from Black Forest Labs.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/flux2-flex`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-flux2-klein-9b`

Flux.2 [klein] 9B model from Black Forest Labs.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/flux2-klein-9b`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-flux2-max`

FLUX.2 [max] delivers state-of-the-art image generation and advanced image editing with exceptional realism, precision, and consistency.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/flux2-max`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-flux2-pro`

Image creation and editing with FLUX.2 [pro] from Black Forest Labs. Ideal for high-quality image manipulation, style transfer, and sequential editing workflows

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/flux2-pro`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-gemini-omni-flash`

Gemini's fast multimodal video model — cinematic clips with native audio from a prompt, a keyframe, or reference images.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/gemini-omni-flash`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-gpt-image-15`

OpenAI-powered image generation with exceptional prompt understanding and versatile editing capabilities.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/gpt-image-15`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-gpt-image-2`

OpenAI's balanced tier; moderate cost and fidelity, ideal for iterative refinement and everyday generation.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/gpt-image-2`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-grok-imagine`

xAI's Grok Imagine image generation model

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/grok-imagine`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-grok-video`

xAI's text-to-video generation model.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/grok-video`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-happy-horse`

Generate video from text with Alibaba Happy Horse 1.0.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/happy-horse`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-hedra-avatar`

Hedra's latest longform avatar model, audio to video will full multi-language support. Perfect for talking and singing video with speaker selection up to 10 minutes long.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/hedra-avatar`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-hedra-character-3`

Hedra's latest longform avatar model, audio to video will full multi-language support. Perfect for talking and singing video with speaker selection up to 10 minutes long.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/hedra-character-3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-hidream-o1-image`

HiDream.ai's open-weights HiDream-O1-Image (8B): one pixel-native model that generates, edits, and personalizes without a VAE or a separate text encoder.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/hidream-o1-image`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-ideogram-v2`

Best in class for poster-ready images and spot-on text rendering in social graphics.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/ideogram-v2`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-ideogram-v4`

Ideogram V4 renders poster-ready text and layout; the required quality parameter picks turbo, balanced or quality, which sets both the render effort and the price.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/ideogram-v4`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-imagen3`

The latest text to image model from Google

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/imagen3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-imagen4`

Google's photoreal model—natural lighting, lifelike skin, and pro-grade sharpness in every shot.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/imagen4`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-16`

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-16`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-21-master`

Cinema-grade video with striking textures and rich depth.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-21-master`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-25-turbo`

Fast, high-quality video generation.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-25-turbo`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-26-motion-control`

Transfer movements from a reference video to any character image. Cost-effective mode for motion transfer, perfect for portraits and simple animations.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-26-motion-control`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-26-pro`

Cinematic visuals, fluid motion, and native audio generation.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-26-pro`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-ai-avatar-v2`

Create avatar videos with realistic humans, animals, cartoons, or stylized characters from an image and audio input.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-ai-avatar-v2`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-o1`

Generate from a single image with text-driven style and scene guidance.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-o1`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-o3`

Text-to-video model with up to 15-second generations and native audio.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-o3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-o3-edit`

Edit videos using natural language.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-o3-edit`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-o3-reference`

Input a reference video and preserve motion and camera style.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-o3-reference`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-v3`

Text-to-video with ultra-high-definition storyboards and native audio.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-v3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-kling-v3-motion-control`

Animate a character image to match the motion of a reference video. Standard tier for cost-effective generation.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/kling-v3-motion-control`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-ltx-2-3`

Lightricks LTX-2.3 text-to-video at up to 4K, with synchronized native audio

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/ltx-2-3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-luma-ray-32`

Luma Ray 3.2 text-to-video with cinematic motion and camera control

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/luma-ray-32`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-mai-image-2-5`

Microsoft AI's MAI-Image-2.5: photorealistic generation and editing with strong in-image typography and design-ready output.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/mai-image-2-5`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-minimax-h3`

MiniMax H3 video generation from text, frames, or references.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/minimax-h3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-minimax-hailuo-02`

Everyday 1080p video with natural movement.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/minimax-hailuo-02`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-minimax-hailuo-23`

Everyday 1080p video with natural movement.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/minimax-hailuo-23`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-minimax-speech-25-hd-preview`

The brand new HD model. Ultimate Similarity, Ultra-High Quality. Supports 40+ languages including Tamil, Hebrew, Swedish, etc.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/minimax-speech-25-hd-preview`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-minimax-speech-25-turbo-preview`

The brand new Turbo model. Ultimate Value, 40 Languages. Major improvements to natural English expression.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/minimax-speech-25-turbo-preview`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-nano-banana`

Best in class image model with reference image support and ultra high quality generations for professional grade images.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/nano-banana`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-nano-banana-2`

Gemini 3.1 Flash native image generation with improved quality and advanced features including multi-subject reference and high-fidelity style transfer

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/nano-banana-2`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-nano-banana-pro`

Gemini 3 Pro native image generation with advanced multimodal understanding and richer visuals

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/nano-banana-pro`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-omnihuman-15`

Creates vivid, emotional character videos driven entirely by your audio.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/omnihuman-15`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-pixverse-v6`

PixVerse V6 text-to-video with native audio and 1080p output up to 15 seconds

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/pixverse-v6`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-qwen-image-2`

Alibaba's Qwen-Image-2.0, tuned for speed. Native 2K output with professional in-image text rendering, for rapid iteration.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/qwen-image-2`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-recraft-v3`

Vector-clean graphics and crisp logos on demand.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/recraft-v3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-reve-21`

Generate images from a text prompt with strong prompt adherence, layout intelligence, and accurate text rendering

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/reve-21`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-reve-21-edit`

Edit one source image from a natural-language instruction, keeping the rest of the image intact

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/reve-21-edit`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-reve-21-remix`

Compose up to eight reference images into a new image from a text prompt

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/reve-21-remix`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-sana`

Lightning-fast and cheap for simple product shots or everyday content.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/sana`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-seedance-15-pro`

ByteDance Seedance 1.5 Pro video generation model

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/seedance-15-pro`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-seedance-20`

ByteDance Seedance 2.0 video generation model

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/seedance-20`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-seedance-20-mini`

ByteDance Seedance 2.0 Mini video generation model

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/seedance-20-mini`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-seedance-25`

ByteDance Seedance 2.5 video generation model

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/seedance-25`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-seedream-40`

Ultra-fast pro grade image model, pairing reference image support with high quality output for professional visuals

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/seedream-40`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-seedream-45`

Latest Seedream with enhanced detail, refined composition, and multi-reference image support for professional visuals.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/seedream-45`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-seedream-50-lite`

ByteDance Seedream 5.0 Lite Text-to-Image

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/seedream-50-lite`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-seedream-50-pro`

ByteDance Seedream 5.0 Pro Text-to-Image

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/seedream-50-pro`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-sora-2-pro`

For complex, narrative-driven videos with remarkable consistency and realistic character-world interaction.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/sora-2-pro`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-veed-fabric-10`

Talking video with natural lip-sync and expressive animation.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/veed-fabric-10`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-veo-2`

The current state of the art in video generation

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/veo-2`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-veo-3`

Hollywood-grade, cinematic video straight from text—your go-to for hero campaigns.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/veo-3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-veo-31`

For unparalleled detail and nuance, perfect for when your vision requires the best possible quality.

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/veo-31`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-vidu-q3`

Vidu Q3 video with native dialogue and sound, up to 16 seconds — from a text prompt, from a start frame, or between a start and end frame

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/vidu-q3`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-vidu-q3-reference`

Vidu Q3 reference-to-video keeping up to four subjects consistent

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/vidu-q3-reference`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra jobs submit-wan-2-7`

Wan 2.7 video with native audio — from a text prompt, from a first frame with an optional last frame, or from reference images that keep subjects consistent

Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.

`POST /models/wan-2-7`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

---

### `hedra keys`

#### `hedra keys create`

Create Key

`POST /keys`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra keys list`

List Keys

`GET /keys`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--workspace-id` | `string` | No | List keys of this workspace; omitted means the authenticating key's workspace. |

#### `hedra keys revoke`

Revoke Key

`DELETE /keys/{key_id}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--key-id` | `string` | Yes | The key's public identifier. |

#### `hedra keys rotate`

Rotate Key

`POST /keys/{key_id}/rotate`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--key-id` | `string` | Yes | The key's public identifier. |
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

---

### `hedra log-drains`

#### `hedra log-drains create-log-drain`

Create Log Drain

`POST /log-drains`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra log-drains delete-log-drain`

Delete Log Drain

`DELETE /log-drains/{drain_id}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--drain-id` | `string` | Yes | The drain's id (`drain_<uuid>`). |

#### `hedra log-drains get-log-drain`

Get Log Drain

`GET /log-drains/{drain_id}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--drain-id` | `string` | Yes | The drain's id (`drain_<uuid>`). |

#### `hedra log-drains list-log-drains`

List Log Drains

`GET /log-drains`

#### `hedra log-drains test-log-drain`

Test Log Drain

`POST /log-drains/{drain_id}/test`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--drain-id` | `string` | Yes | The drain's id (`drain_<uuid>`). |

#### `hedra log-drains update-log-drain`

Update Log Drain

`PATCH /log-drains/{drain_id}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--drain-id` | `string` | Yes | The drain's id (`drain_<uuid>`). |
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

---

### `hedra models`

#### `hedra models estimate`

Estimate

`POST /models/{model}/estimate`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes | The model's public id (`GET /v3/models`). |
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra models get`

Get Model

`GET /models/{model}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes | The model's public id (`GET /v3/models`). |

#### `hedra models get-openapi`

A standalone one-operation OpenAPI spec for this model's submit call.

`GET /models/{model}/openapi.json`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes | The model's public id (`GET /v3/models`). |

#### `hedra models list`

List Models

`GET /models`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--modality` | `string` | No | Only models with this modality, matching `modality` on each returned model. |

#### `hedra models list-model-jobs`

List Model Jobs

`GET /models/{model}/jobs`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes | The model's public id (`GET /v3/models`). |
| `--limit` | `integer` | No | Maximum items per page. |
| `--cursor` | `string` | No | Opaque cursor from the previous page's `next_cursor`; omit for the first page. |

#### `hedra models list-voices`

Voices this model accepts — scoped to the model's voice provider. fern-config end-to-end regeneration probe 20260811-050507.

`GET /models/{model}/voices`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes | The model's public id (`GET /v3/models`). |

---

### `hedra tokens`

#### `hedra tokens create`

Create Token

`POST /tokens`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

---

### `hedra webhooks`

#### `hedra webhooks delete-default`

Delete Default

`DELETE /webhooks/default`

#### `hedra webhooks get-default`

Get Default

`GET /webhooks/default`

#### `hedra webhooks get-public-key`

Public Key

`GET /webhooks/public-key`

#### `hedra webhooks list-deliveries`

List Deliveries

`GET /webhooks/deliveries`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--limit` | `integer` | No | Maximum items per page. |
| `--cursor` | `string` | No | Opaque cursor from the previous page's `next_cursor`; omit for the first page. |

#### `hedra webhooks put-default`

Put Default

`PUT /webhooks/default`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra webhooks redeliver`

Replay a finished delivery: reset it to PENDING and re-fire the signed POST.

404 if the delivery isn't visible to the caller; 409 if a delivery for the
request is still in flight (a replay must not stack on it). The delivery is
re-validated (SSRF) and re-signed at send time, and the receiver dedupes on
``X-Hedra-Webhook-Id``, so a replay is safe.

The webhook id is stable across the original and every replay, because it
identifies the event. Every attempt of a replayed cycle therefore also carries
``X-Hedra-Webhook-Redelivery: true`` — without it a receiver doing exactly what
our guidance says (dedupe on the id) would silently discard the replay, which is
the one case where the duplicate is the point.

`POST /webhooks/deliveries/{job_id}/redeliver`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--job-id` | `string` | Yes | The job's id (`job_<uuid>`). |

#### `hedra webhooks test-default`

Test Default

`POST /webhooks/default/test`

---

## Global flags

These flags are available on every command:

| Flag | Description |
|------|-------------|
| `--dry-run` | Print the HTTP request without sending it |
| `--json <JSON\|->` | Supply the request body as JSON (or `-` for stdin) |
| `--params <JSON>` | Merge extra parameters as JSON |
| `--format <json\|table\|yaml\|csv>` | Output format (default: `json`) |
| `--output <PATH>` | Write binary responses to a file |
| `--base-url <URL>` | Override the API base URL |
| `--page-all` | Auto-paginate and stream all results |
| `--page-limit <N>` | Max pages to fetch (default: `10`) |
| `-q, --quiet` | Suppress stdout on success |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

