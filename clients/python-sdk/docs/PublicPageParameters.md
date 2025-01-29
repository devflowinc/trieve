# PublicPageParameters


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**allow_switching_modes** | **bool** |  | [optional] 
**analytics** | **bool** |  | [optional] 
**api_key** | **str** |  | [optional] 
**base_url** | **str** |  | [optional] 
**brand_color** | **str** |  | [optional] 
**brand_font_family** | **str** |  | [optional] 
**brand_logo_img_src_url** | **str** |  | [optional] 
**brand_name** | **str** |  | [optional] 
**button_triggers** | [**List[ButtonTrigger]**](ButtonTrigger.md) |  | [optional] 
**chat** | **bool** |  | [optional] 
**creator_linked_in_url** | **str** |  | [optional] 
**creator_name** | **str** |  | [optional] 
**currency_position** | **str** |  | [optional] 
**dataset_id** | **str** |  | [optional] 
**debounce_ms** | **int** |  | [optional] 
**default_ai_questions** | **List[str]** |  | [optional] 
**default_currency** | **str** |  | [optional] 
**default_image_question** | **str** |  | [optional] 
**default_search_mode** | **str** |  | [optional] 
**default_search_queries** | **List[str]** |  | [optional] 
**floating_button_position** | **str** |  | [optional] 
**floating_search_icon_position** | **str** |  | [optional] 
**followup_questions** | **bool** |  | [optional] 
**for_brand_name** | **str** |  | [optional] 
**heading_prefix** | **str** |  | [optional] 
**hero_pattern** | [**HeroPattern**](HeroPattern.md) |  | [optional] 
**hide_drawn_text** | **bool** |  | [optional] 
**inline** | **bool** |  | [optional] 
**inline_header** | **str** |  | [optional] 
**is_test_mode** | **bool** |  | [optional] 
**nav_logo_img_src_url** | **str** |  | [optional] 
**number_of_suggestions** | **int** |  | [optional] 
**open_graph_metadata** | [**OpenGraphMetadata**](OpenGraphMetadata.md) |  | [optional] 
**open_links_in_new_tab** | **bool** |  | [optional] 
**placeholder** | **str** |  | [optional] 
**problem_link** | **str** |  | [optional] 
**responsive** | **bool** |  | [optional] 
**search_options** | [**PublicPageSearchOptions**](PublicPageSearchOptions.md) |  | [optional] 
**show_floating_button** | **bool** |  | [optional] 
**show_floating_input** | **bool** |  | [optional] 
**show_floating_search_icon** | **bool** |  | [optional] 
**single_product_options** | [**SingleProductOptions**](SingleProductOptions.md) |  | [optional] 
**suggested_queries** | **bool** |  | [optional] 
**tab_messages** | [**List[PublicPageTabMessage]**](PublicPageTabMessage.md) |  | [optional] 
**tags** | [**List[PublicPageTag]**](PublicPageTag.md) |  | [optional] 
**theme** | [**PublicPageTheme**](PublicPageTheme.md) |  | [optional] 
**type** | **str** |  | [optional] 
**use_group_search** | **bool** |  | [optional] 
**use_local** | **bool** |  | [optional] 
**use_pagefind** | **bool** |  | [optional] 
**video_link** | **str** |  | [optional] 
**video_position** | **str** |  | [optional] 
**z_index** | **int** |  | [optional] 

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


