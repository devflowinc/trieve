# ConditionType

Filters can be constructed using either fields on the chunk objects, ids or tracking ids of chunks, and finally ids or tracking ids of groups.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**boolean** | **bool** | Boolean is a true false value for a field. This only works for boolean fields. You can specify this if you want values to be true or false. | [optional] 
**date_range** | [**DateRange**](DateRange.md) |  | [optional] 
**field** | **str** | Field is the name of the field to filter on. Commonly used fields are &#x60;timestamp&#x60;, &#x60;link&#x60;, &#x60;tag_set&#x60;, &#x60;location&#x60;, &#x60;num_value&#x60;, &#x60;group_ids&#x60;, and &#x60;group_tracking_ids&#x60;. The field value will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. To access fields inside of the metadata that you provide with the card, prefix the field name with &#x60;metadata.&#x60;. | 
**geo_bounding_box** | [**LocationBoundingBox**](LocationBoundingBox.md) |  | [optional] 
**geo_polygon** | [**LocationPolygon**](LocationPolygon.md) |  | [optional] 
**geo_radius** | [**LocationRadius**](LocationRadius.md) |  | [optional] 
**match_all** | [**List[MatchCondition]**](MatchCondition.md) | Match all lets you pass in an array of values that will return results if all of the items match. The match value will be used to check for an exact substring match on the metadata values for each existing chunk. If both match_all and match_any are provided, the match_any condition will be used. | [optional] 
**match_any** | [**List[MatchCondition]**](MatchCondition.md) | Match any lets you pass in an array of values that will return results if any of the items match. The match value will be used to check for an exact substring match on the metadata values for each existing chunk. If both match_all and match_any are provided, the match_any condition will be used. | [optional] 
**range** | [**Range**](Range.md) |  | [optional] 
**ids** | **List[str]** | Ids of the chunks to apply a match_any condition with. Only chunks with one of these ids will be returned. | [optional] 
**tracking_ids** | **List[str]** | Tracking ids of the chunks to apply a match_any condition with. Only chunks with one of these tracking ids will be returned. | [optional] 

## Example

```python
from trieve_py_client.models.condition_type import ConditionType

# TODO update the JSON string below
json = "{}"
# create an instance of ConditionType from a JSON string
condition_type_instance = ConditionType.from_json(json)
# print the JSON string representation of the object
print(ConditionType.to_json())

# convert the object into a dict
condition_type_dict = condition_type_instance.to_dict()
# create an instance of ConditionType from a dict
condition_type_form_dict = condition_type.from_dict(condition_type_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


