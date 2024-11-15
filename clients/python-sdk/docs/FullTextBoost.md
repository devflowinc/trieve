# FullTextBoost

Boost the presence of certain tokens for fulltext (SPLADE) and keyword (BM25) search. I.e. boosting title phrases to priortize title matches or making sure that the listing for AirBNB itself ranks higher than companies who make software for AirBNB hosts by boosting the in-document-frequency of the AirBNB token (AKA word) for its official listing. Conceptually it multiples the in-document-importance second value in the tuples of the SPLADE or BM25 sparse vector of the chunk_html innerText for all tokens present in the boost phrase by the boost factor like so: (token, in-document-importance) -> (token, in-document-importance*boost_factor).

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**boost_factor** | **float** | Amount to multiplicatevly increase the frequency of the tokens in the phrase by | 
**phrase** | **str** | The phrase to boost in the fulltext document frequency index | 

## Example

```python
from trieve_py_client.models.full_text_boost import FullTextBoost

# TODO update the JSON string below
json = "{}"
# create an instance of FullTextBoost from a JSON string
full_text_boost_instance = FullTextBoost.from_json(json)
# print the JSON string representation of the object
print(FullTextBoost.to_json())

# convert the object into a dict
full_text_boost_dict = full_text_boost_instance.to_dict()
# create an instance of FullTextBoost from a dict
full_text_boost_form_dict = full_text_boost.from_dict(full_text_boost_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


