# ScrapeOptionsOneOf


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**openapi_schema_url** | **str** | OpenAPI json schema to be processed alongside the site crawl | 
**openapi_tag** | **str** | Tag to look for to determine if a page should create an openapi route chunk instead of chunks from heading-split of the HTML | 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.scrape_options_one_of import ScrapeOptionsOneOf

# TODO update the JSON string below
json = "{}"
# create an instance of ScrapeOptionsOneOf from a JSON string
scrape_options_one_of_instance = ScrapeOptionsOneOf.from_json(json)
# print the JSON string representation of the object
print(ScrapeOptionsOneOf.to_json())

# convert the object into a dict
scrape_options_one_of_dict = scrape_options_one_of_instance.to_dict()
# create an instance of ScrapeOptionsOneOf from a dict
scrape_options_one_of_form_dict = scrape_options_one_of.from_dict(scrape_options_one_of_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


