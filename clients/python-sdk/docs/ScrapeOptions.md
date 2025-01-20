# ScrapeOptions

Options for including an openapi spec or shopify settigns

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**openapi_schema_url** | **str** | OpenAPI json schema to be processed alongside the site crawl | 
**openapi_tag** | **str** | Tag to look for to determine if a page should create an openapi route chunk instead of chunks from heading-split of the HTML | 
**type** | **str** |  | 
**group_variants** | **bool** | This option will ingest all variants as individual chunks and place them in groups by product id. Turning this off will only scrape 1 variant per product. default: true | [optional] 
**tag_regexes** | **List[str]** |  | [optional] 

## Example

```python
from trieve_py_client.models.scrape_options import ScrapeOptions

# TODO update the JSON string below
json = "{}"
# create an instance of ScrapeOptions from a JSON string
scrape_options_instance = ScrapeOptions.from_json(json)
# print the JSON string representation of the object
print(ScrapeOptions.to_json())

# convert the object into a dict
scrape_options_dict = scrape_options_instance.to_dict()
# create an instance of ScrapeOptions from a dict
scrape_options_form_dict = scrape_options.from_dict(scrape_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


