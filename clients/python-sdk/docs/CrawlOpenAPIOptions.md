# CrawlOpenAPIOptions

Options for including an openapi spec in the crawl

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**openapi_schema_url** | **str** | OpenAPI json schema to be processed alongside the site crawl | 
**openapi_tag** | **str** | Tag to look for to determine if a page should create an openapi route chunk instead of chunks from heading-split of the HTML | 

## Example

```python
from trieve_py_client.models.crawl_open_api_options import CrawlOpenAPIOptions

# TODO update the JSON string below
json = "{}"
# create an instance of CrawlOpenAPIOptions from a JSON string
crawl_open_api_options_instance = CrawlOpenAPIOptions.from_json(json)
# print the JSON string representation of the object
print(CrawlOpenAPIOptions.to_json())

# convert the object into a dict
crawl_open_api_options_dict = crawl_open_api_options_instance.to_dict()
# create an instance of CrawlOpenAPIOptions from a dict
crawl_open_api_options_form_dict = crawl_open_api_options.from_dict(crawl_open_api_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


