# EventData

EventData represents a single analytics event

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **str** | The time the event was created. | 
**dataset_id** | **str** | The unique identifier for the dataset the event is associated with. | 
**event_name** | **str** | The name of the event, e.g. \&quot;Added to Cart\&quot;, \&quot;Purchased\&quot;, \&quot;Viewed Home Page\&quot;, \&quot;Clicked\&quot;, \&quot;Filter Clicked\&quot;. | 
**event_type** | **str** | The type of event, \&quot;add_to_cart\&quot;, \&quot;purchase\&quot;, \&quot;view\&quot;, \&quot;click\&quot;, \&quot;filter_clicked\&quot;. | 
**id** | **str** | The unique identifier for the event | 
**is_conversion** | **bool** | Whether the event is a conversion event. | [optional] 
**items** | **List[str]** | The items associated with the event. This could be a list of stringified json chunks for search events, or a list of items for add_to_cart, purchase, view, and click events. | 
**metadata** | **object** | Additional metadata associated with the event. This can be custom data that is specific to the event. | [optional] 
**request_id** | **str** | The unique identifier for the request the event is associated with. | [optional] 
**request_type** | **str** | The type of request the event is associated with. | [optional] 
**updated_at** | **str** | The time the event was last updated. | 
**user_id** | **str** | The user identifier associated with the event. | [optional] 

## Example

```python
from trieve_py_client.models.event_data import EventData

# TODO update the JSON string below
json = "{}"
# create an instance of EventData from a JSON string
event_data_instance = EventData.from_json(json)
# print the JSON string representation of the object
print(EventData.to_json())

# convert the object into a dict
event_data_dict = event_data_instance.to_dict()
# create an instance of EventData from a dict
event_data_form_dict = event_data.from_dict(event_data_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


