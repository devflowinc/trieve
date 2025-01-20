# ScrapeOptionsOneOf1


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**group_variants** | **bool** | This option will ingest all variants as individual chunks and place them in groups by product id. Turning this off will only scrape 1 variant per product. default: true | [optional] 
**tag_regexes** | **List[str]** |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.scrape_options_one_of1 import ScrapeOptionsOneOf1

# TODO update the JSON string below
json = "{}"
# create an instance of ScrapeOptionsOneOf1 from a JSON string
scrape_options_one_of1_instance = ScrapeOptionsOneOf1.from_json(json)
# print the JSON string representation of the object
print(ScrapeOptionsOneOf1.to_json())

# convert the object into a dict
scrape_options_one_of1_dict = scrape_options_one_of1_instance.to_dict()
# create an instance of ScrapeOptionsOneOf1 from a dict
scrape_options_one_of1_form_dict = scrape_options_one_of1.from_dict(scrape_options_one_of1_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


