# HeroPattern


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**background_color** | **str** |  | [optional] 
**foreground_color** | **str** |  | [optional] 
**foreground_opacity** | **float** |  | [optional] 
**hero_pattern_name** | **str** |  | [optional] 
**hero_pattern_svg** | **str** |  | [optional] 

## Example

```python
from trieve_py_client.models.hero_pattern import HeroPattern

# TODO update the JSON string below
json = "{}"
# create an instance of HeroPattern from a JSON string
hero_pattern_instance = HeroPattern.from_json(json)
# print the JSON string representation of the object
print(HeroPattern.to_json())

# convert the object into a dict
hero_pattern_dict = hero_pattern_instance.to_dict()
# create an instance of HeroPattern from a dict
hero_pattern_form_dict = hero_pattern.from_dict(hero_pattern_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


