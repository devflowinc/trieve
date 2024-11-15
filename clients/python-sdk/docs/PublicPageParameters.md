# PublicPageParameters


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**allow_switching_modes** | **bool** |  | [optional] 
**analytics** | **bool** |  | [optional] 
**api_key** | **str** |  | [optional] 
**base_url** | **str** |  | [optional] 
**brand_color** | **str** |  | [optional] 
**brand_logo_img_src_url** | **str** |  | [optional] 
**brand_name** | **str** |  | [optional] 
**chat** | **bool** |  | [optional] 
**currency_position** | **str** |  | [optional] 
**dataset_id** | **str** |  | [optional] 
**debounce_ms** | **int** |  | [optional] 
**default_ai_questions** | **List[str]** |  | [optional] 
**default_currency** | **str** |  | [optional] 
**default_search_mode** | **str** |  | [optional] 
**default_search_queries** | **List[str]** |  | [optional] 
**placeholder** | **str** |  | [optional] 
**problem_link** | **str** |  | [optional] 
**responsive** | **bool** |  | [optional] 
**search_options** | [**PublicPageSearchOptions**](PublicPageSearchOptions.md) |  | [optional] 
**suggested_queries** | **bool** |  | [optional] 
**theme** | [**PublicPageTheme**](PublicPageTheme.md) |  | [optional] 
**type** | **str** |  | [optional] 
**use_group_search** | **bool** |  | [optional] 

## Example

```python
from trieve_py_client.models.public_page_parameters import PublicPageParameters

# TODO update the JSON string below
json = "{}"
# create an instance of PublicPageParameters from a JSON string
public_page_parameters_instance = PublicPageParameters.from_json(json)
# print the JSON string representation of the object
print(PublicPageParameters.to_json())

# convert the object into a dict
public_page_parameters_dict = public_page_parameters_instance.to_dict()
# create an instance of PublicPageParameters from a dict
public_page_parameters_form_dict = public_page_parameters.from_dict(public_page_parameters_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


