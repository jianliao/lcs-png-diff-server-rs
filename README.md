# lcs-png-diff-server
Pass in before and after bitmap URL, then the server will generate lcs diff png and return the diff result URL in response.

## Example

### Request payload

``` bash
curl \
  -d '{
        "before_png": "http://localhost:3000/fixtures/slider.png",
        "after_png": "http://localhost:3000/fixtures/slider_after.png"
      }' \
  -H 'Content-Type: application/json' \
  -X POST http://localhost:8080/api/diff
```

### Response payload

``` json
{
  "result_url": "http://localhost:8080/assets/b02d9094-bc6c-4c40-923e-50c66bcf1951.png"
}
```

## LICENSE

Apache License Version 2.0