# CrawlOptions


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**exclude_paths** | **List[str]** | URL Patterns to exclude from the crawl | [optional] 
**exclude_tags** | **List[str]** | Specify the HTML tags, classes and ids to exclude from the response. | [optional] 
**include_paths** | **List[str]** | URL Patterns to include in the crawl | [optional] 
**include_tags** | **List[str]** | Specify the HTML tags, classes and ids to include in the response. | [optional] 
**interval** | [**CrawlInterval**](CrawlInterval.md) |  | [optional] 
**limit** | **int** | How many pages to crawl, defaults to 20 | [optional] 
**max_depth** | **int** | How many levels deep to crawl, defaults to 2 | [optional] 
**site_url** | **str** | The URL to crawl | [optional] 

## Example

```python
from trieve_py_client.models.crawl_options import CrawlOptions

# TODO update the JSON string below
json = "{}"
# create an instance of CrawlOptions from a JSON string
crawl_options_instance = CrawlOptions.from_json(json)
# print the JSON string representation of the object
print(CrawlOptions.to_json())

# convert the object into a dict
crawl_options_dict = crawl_options_instance.to_dict()
# create an instance of CrawlOptions from a dict
crawl_options_form_dict = crawl_options.from_dict(crawl_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


