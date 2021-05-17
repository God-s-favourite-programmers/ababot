import datetime

class Event():
    def __init__(self, name:str, description:str, event_time:datetime.datetime, event_location:str, registration_open:datetime.datetime, url:str) -> None:
        self.name:str = name
        self.description:str = description
        self.event_time:datetime.datetime = event_time
        self.event_location:str = event_location
        if registration_open == None:
            raise ValueError("registration_open must not be None")
        self.registration_open:datetime.datetime = registration_open
        self.url:str = url
    
    def get_name(self) -> str:
        return self.name

    def get_description(self) -> str:
        return self.description
    
    def get_event_time(self) -> datetime.datetime:
        return self.event_time

    def get_event_location(self) -> str:
        return self.event_location

    def get_registration_open(self) -> str:
        return self.registration_open

    def get_url(self) -> str:
        return self.url